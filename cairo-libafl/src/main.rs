use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
mod args;
use args::Opt;
mod utils;
mod cairo_vm;
use clap::Parser;
use utils::parse_json::parse_json;
use cairo_vm::cairo_runner::runner;
use std::{fs, path::PathBuf};
use libafl::prelude::MultiMonitor;
use libafl::prelude::RandPrintablesGenerator;
use libafl::{
    bolts::{
        current_nanos,
        launcher::Launcher,
        rands::StdRand,
        shmem::{ShMemProvider, StdShMemProvider},
        tuples::tuple_list,
    },
    corpus::{InMemoryCorpus, OnDiskCorpus},
    events::EventConfig,
    executors::{inprocess::InProcessExecutor, ExitKind},
    feedbacks::{CrashFeedback, MaxMapFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    inputs::BytesInput,
    mutators::scheduled::{havoc_mutations, StdScheduledMutator},
    observers::StdMapObserver,
    schedulers::QueueScheduler,
    stages::mutational::StdMutationalStage,
    state::StdState,
    Error,
};


/// Coverage map with explicit assignments due to the lack of instrumentation
static mut SIGNALS: [u8; 16] = [0; 16];

/// Assign a signal to the signals map
fn signals_set(idx: usize) {
    unsafe { SIGNALS[idx] = 1 };
}

pub fn main() {
    let opt = Opt::parse();

    let cores = opt.cores;
    let contract = opt
        .contract
        .to_str()
        .expect("Fuzzer needs path to contract");
    let iter = opt.iteration;

    let shmem_provider = StdShMemProvider::new().expect("Failed to init shared memory");

    let monitor = MultiMonitor::new(|s| println!("{}", s));
    
    let functions = parse_json(&contract.to_string());
    let contents =
        fs::read_to_string(&contract.to_string()).expect("Should have been able to read the file");
    let mut run_client = |_state: Option<_>, mut mgr, _core_id| {
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

        // The wrapped harness function
        let mut harness = |_input: &BytesInput| {
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
        if iter == -1 {
            fuzzer
                .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
                .expect("Error in the fuzzing loop");
        } else {
            fuzzer
                .fuzz_loop_for(
                    &mut stages,
                    &mut executor,
                    &mut state,
                    &mut mgr,
                    iter as u64,
                )
                .expect("Error in the fuzzing loop");
        }
        Ok(())
    };

    match Launcher::builder()
        .shmem_provider(shmem_provider)
        .configuration(EventConfig::from_name("default"))
        .monitor(monitor)
        .run_client(&mut run_client)
        .cores(&cores)
        .stdout_file(Some("/dev/null"))
        .build()
        .launch()
    {
        Ok(()) => (),
        Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Good bye."),
        Err(err) => panic!("Failed to run launcher: {:?}", err),
    }
}
