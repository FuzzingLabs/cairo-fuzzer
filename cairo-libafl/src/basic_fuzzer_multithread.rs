//! A libfuzzer-like fuzzer with llmp-multithreading support and restarts
//! The example harness is built for libpng.
//! In this example, you will see the use of the `launcher` feature.
//! The `launcher` will spawn new processes for each cpu core.
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use core::time::Duration;
use std::{env, net::SocketAddr, path::PathBuf};

use clap::{self, Parser};
use libafl::prelude::RandPrintablesGenerator;
use libafl::prelude::SimpleEventManager;
use libafl::prelude::SimpleMonitor;
use libafl::{
    bolts::{
        core_affinity::Cores,
        current_nanos,
        launcher::Launcher,
        rands::StdRand,
        shmem::{ShMemProvider, StdShMemProvider},
        tuples::{tuple_list, Merge},
        AsSlice,
    },
    corpus::{Corpus, InMemoryCorpus, OnDiskCorpus},
    events::EventConfig,
    executors::{inprocess::InProcessExecutor, ExitKind, TimeoutExecutor},
    feedback_or, feedback_or_fast,
    feedbacks::{CrashFeedback, MaxMapFeedback, TimeFeedback, TimeoutFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    inputs::{BytesInput, HasTargetBytes},
    monitors::{MultiMonitor, OnDiskTOMLMonitor},
    mutators::{
        scheduled::{havoc_mutations, tokens_mutations, StdScheduledMutator},
        token_mutations::Tokens,
    },
    observers::{HitcountsMapObserver, StdMapObserver, TimeObserver},
    schedulers::{IndexesLenTimeMinimizerScheduler, QueueScheduler},
    stages::mutational::StdMutationalStage,
    state::{HasCorpus, HasMetadata, StdState},
    Error,
};
use libafl_targets::{libfuzzer_initialize, libfuzzer_test_one_input, EDGES_MAP, MAX_EDGES_NUM};

use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::types::program::Program;
use cairo_rs::types::relocatable::MaybeRelocatable;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;
use num_bigint::BigInt;
use num_bigint::Sign;
use std::any::Any;
use std::fs;
use std::path::Path;
mod parse_json;
use crate::parse_json::parse_json;
mod utils;

fn timeout_from_millis_str(time: &str) -> Result<Duration, Error> {
    Ok(Duration::from_millis(time.parse()?))
}

#[derive(Debug, Parser)]
struct Opt {
    #[arg(
        short,
        long,
        value_parser = Cores::from_cmdline,
        help = "Spawn a client in each of the provided cores. Broker runs in the 0th core. 'all' to select all available cores. 'none' to run a client without binding to any core. eg: '1,2-4,6' selects the cores 1,2,3,4,6.",
        name = "CORES"
    )]
    cores: Cores,

    #[arg(
        short = 'p',
        long,
        help = "Choose the broker TCP port, default is 1337",
        name = "PORT",
        default_value = "1337"
    )]
    broker_port: u16,

    #[arg(short = 'a', long, help = "Specify a remote broker", name = "REMOTE")]
    remote_broker_addr: Option<SocketAddr>,

    #[arg(short, long, help = "Set an initial corpus directory", name = "INPUT")]
    input: Vec<PathBuf>,

    #[arg(
        short,
        long,
        help = "Set the output directory, default is ./out",
        name = "OUTPUT",
        default_value = "./out"
    )]
    output: PathBuf,

    #[arg(
        value_parser = timeout_from_millis_str,
        short,
        long,
        help = "Set the exeucution timeout in milliseconds, default is 10000",
        name = "TIMEOUT",
        default_value = "10000"
    )]
    timeout: Duration,
}

/// Coverage map with explicit assignments due to the lack of instrumentation
static mut SIGNALS: [u8; 16] = [0; 16];

/// Assign a signal to the signals map
fn signals_set(idx: usize) {
    unsafe { SIGNALS[idx] = 1 };
}

fn runner(json: &String, func_name: String, args_num: u64, data: isize) {
    //println!("====> Running function : {}", func_name);
    //println!("");
    let program = Program::from_string(json, Some(&func_name)).unwrap();
    let mut cairo_runner = cairo_runner!(program);
    let mut vm = vm!();
    let hint_processor = BuiltinHintProcessor::new_empty();

    let entrypoint = match program
        .identifiers
        .get(&format!("__main__.{}", &func_name))
        .unwrap()
        .pc
    {
        Some(value) => value,
        None => return,
    };

    cairo_runner.initialize_builtins(&mut vm).unwrap();
    cairo_runner.initialize_segments(&mut vm, None);
    let value = &MaybeRelocatable::from((data, 0));
    let mut args = Vec::<&dyn Any>::new();
    args.push(value);
    for _i in 0..args_num {
        args.push(value);
    }
    let _var = cairo_runner.run_from_entrypoint(
        entrypoint,
        args,
        false,
        true,
        true,
        &mut vm,
        &hint_processor,
    );

    let mut stdout = Vec::<u8>::new();
    cairo_runner.write_output(&mut vm, &mut stdout).unwrap();
    /*println!("write output : {:?}", stdout);
    println!("");
    println!(
        "get output : {:?}",
        cairo_runner.get_output(&mut vm).unwrap()
    );
    println!("");*/
}

pub fn main() {
    let opt = Opt::parse();

    let broker_port = opt.broker_port;
    let cores = opt.cores;

    println!(
        "Workdir: {:?}",
        env::current_dir().unwrap().to_string_lossy().to_string()
    );

    let shmem_provider = StdShMemProvider::new().expect("Failed to init shared memory");

    let monitor = OnDiskTOMLMonitor::new(
        "./fuzzer_stats.toml",
        MultiMonitor::new(|s| println!("{}", s)),
    );

    let mut run_client = |state: Option<_>, mut mgr, _core_id| {
        let observer =
            unsafe { StdMapObserver::new_from_ptr("signals", SIGNALS.as_mut_ptr(), SIGNALS.len()) };

        // Feedback to rate the interestingness of an input
        let mut feedback = MaxMapFeedback::new(&observer);

        // A feedback to choose if an input is a solution or not
        let mut objective = CrashFeedback::new();

        // create a State from scratch
        let mut state = StdState::new(
            // RNG
            StdRand::with_seed(current_nanos()),
            // Corpus that will be evolved, we keep it in memory for performance
            InMemoryCorpus::new(),
            // Corpus in which we store solutions (crashes in this example),
            // on disk so the user can get them after stopping the fuzzer
            OnDiskCorpus::new(PathBuf::from("./crashes")).unwrap(),
            // States of the feedbacks.
            // The feedbacks can report the data that should persist in the State.
            &mut feedback,
            // Same for objective feedbacks
            &mut objective,
        )
        .unwrap();

        // A queue policy to get testcasess from the corpus
        let scheduler = QueueScheduler::new();

        // A fuzzer with feedbacks and a corpus scheduler
        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

        let functions = parse_json(&"json/vuln.json".to_string());
        let contents = fs::read_to_string(&"json/vuln.json".to_string())
            .expect("Should have been able to read the file");
        // The wrapped harness function, calling out to the LLVM-style harness
        let mut harness = |input: &BytesInput| {
            for function in functions.clone() {
                signals_set(1); // set SIGNALS[1]
                runner(&contents, function.name, function.num_args, 2);
                signals_set(2); // set SIGNALS[2]
            }
            ExitKind::Ok
        };

        // Create the executor for an in-process function with just one observer
        let mut executor = InProcessExecutor::new(
            &mut harness,
            tuple_list!(observer),
            &mut fuzzer,
            &mut state,
            &mut mgr,
        )?;

        // Generator of printable bytearrays of max size 32
        let mut generator = RandPrintablesGenerator::new(32);

        // Generate 8 initial inputs
        state
            .generate_initial_inputs(&mut fuzzer, &mut executor, &mut generator, &mut mgr, 8)
            .expect("Failed to generate the initial corpus");

        // Setup a mutational stage with a basic bytes mutator
        let mutator = StdScheduledMutator::new(havoc_mutations());
        let mut stages = tuple_list!(StdMutationalStage::new(mutator));
        fuzzer
            .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
            .expect("Error in the fuzzing loop");
        Ok(())
    };


    match Launcher::builder()
        .shmem_provider(shmem_provider)
        .configuration(EventConfig::from_name("default"))
        .monitor(monitor)
        .run_client(&mut run_client)
        .cores(&cores)
        .broker_port(broker_port)
        .remote_broker_addr(opt.remote_broker_addr)
        .stdout_file(Some("/dev/null"))
        .build()
        .launch()
    {
        Ok(()) => (),
        Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Good bye."),
        Err(err) => panic!("Failed to run launcher: {:?}", err),
    }
}
