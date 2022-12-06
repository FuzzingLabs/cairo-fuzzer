use libafl::prelude::AsSlice;
use libafl::prelude::HasTargetBytes;
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
mod args;
use args::Opt;
mod cairo_vm;
mod utils;
use cairo_vm::cairo_runner::runner;
use clap::Parser;
mod input_generator;
use debug_print::debug_println as dprintln;
use input_generator::generator::MyRandPrintablesGenerator;
use libafl::prelude::SimpleMonitor;
use libafl::prelude::*;
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
    mutators::scheduled::StdScheduledMutator,
    observers::StdMapObserver,
    schedulers::QueueScheduler,
    stages::mutational::StdMutationalStage,
    state::StdState,
    Error,
};
use std::{fs, path::PathBuf};
use utils::parse_json::parse_json;

/// Coverage map with explicit assignments due to the lack of instrumentation
static mut SIGNALS_FP: [usize; 100000] = [0; 100000];
static mut SIGNALS_PC: [usize; 100000] = [0; 100000];
//static mut SIGNALS: [usize; 1000000] = [0; 1000000];


/// Assign a signal to the signals map
fn signals_set(fp: usize, pc: usize) {
    unsafe { SIGNALS_FP[fp] = 1 };
    unsafe { SIGNALS_PC[pc] = 1 };
    //unsafe { SIGNALS[sig] = 1 };
}

pub fn debug_input(buf: &[u8]) {
    dprintln!("----");
    if buf[0] as char == 'f' {
        dprintln!("f");

        if buf[1] as char == 'u' {
            dprintln!("u");

            if buf[2] as char == 'z' {
                dprintln!("z");

                if buf[3] as char == 'z' {
                    dprintln!("z");

                    if buf[4] as char == 'i' {
                        dprintln!("i");

                        if buf[5] as char == 'n' {
                            dprintln!("n");

                            if buf[6] as char == 'g' {
                                dprintln!("g");
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn main() {
    let opt = Opt::parse();
    let cores = opt.cores;
    let contract = opt
        .contract
        .to_str()
        .expect("Fuzzer needs path to contract");
    let iter = opt.iteration;
    let function_name = opt.function;

    let shmem_provider = StdShMemProvider::new().expect("Failed to init shared memory");
    let monitor = SimpleMonitor::new(|s| println!("{}", s));
    let contents =
        fs::read_to_string(&contract.to_string()).expect("Should have been able to read the file");
    let function = match parse_json(&contents, &function_name) {
        Some(func) => func,
        None => {
            println!("Could not find the function {}", function_name);
            return;
        }
    };

    let mut run_client = |_state: Option<_>, mut mgr, _core_id| {
        let observer_fp = unsafe {
            StdMapObserver::new_from_ptr("signals_fp", SIGNALS_FP.as_mut_ptr(), SIGNALS_FP.len())
        };

        let observer_pc = unsafe {
            StdMapObserver::new_from_ptr("signals_fp", SIGNALS_PC.as_mut_ptr(), SIGNALS_PC.len())
        };
        //let observer = unsafe {
        //    StdMapObserver::new_from_ptr("signals", SIGNALS.as_mut_ptr(), SIGNALS.len())
        //};
        let mut feedback = feedback_or!(
            MaxMapFeedback::new(&observer_pc),
            MaxMapFeedback::new(&observer_fp)
        );
        //let mut feedback = MaxMapFeedback::new(&observer);
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

        // A queue policy to get testcases from the corpus
        let scheduler = QueueScheduler::new();

        // A fuzzer with feedbacks and a corpus scheduler
        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);
        let mut harness = |input: &BytesInput| {
            let target = input.target_bytes();
            let buf = target.as_slice();
            if !buf.is_empty() && buf.len() == 11 {
                //debug_input(buf);
                match runner(&contents, function.name.clone(), input) {
                    Ok(traces) => {
                        for trace in traces.unwrap() {
                            signals_set(trace.0.offset, trace.1.offset);
                            //dprintln!("Setting signals! {} {}",trace.0.offset, trace.1.offset);
                        }
                    }
                    Err(_e) => (),
                }
            } else {
                println!("BUG IN GENERATOR {}", buf.len());
            }
            ExitKind::Ok
        };

        // Create the executor for an in-process function with just one observer
        let mut executor = InProcessExecutor::new(
            &mut harness,
            tuple_list!(observer_fp, observer_pc),
            //tuple_list!(observer),
            &mut fuzzer,
            &mut state,
            &mut mgr,
        )?;

        // Generator of printable bytearrays of max size 32
        let mut generator = MyRandPrintablesGenerator::new(11);

        // Generate 8 initial inputs
        state
            .generate_initial_inputs(&mut fuzzer, &mut executor, &mut generator, &mut mgr, 8)
            .expect("Failed to generate the initial corpus");

        // Setup a mutational stage with a basic bytes mutator
        let mutator = StdScheduledMutator::new(tuple_list!(BitFlipMutator::new()));
        let mut stages = tuple_list!(StdMutationalStage::new(mutator));
        if let Some(iters) = iter {
            println!("Running {} iter", {iters});
            fuzzer
            .fuzz_loop_for(
                &mut stages,
                &mut executor,
                &mut state,
                &mut mgr,
                iters,
            )
            .expect("Error in the fuzzing loop");
        } else {
            fuzzer
                .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
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
        //.stdout_file(Some("/dev/null"))
        .build()
        .launch()
    {
        Ok(()) => (),
        Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Good bye."),
        Err(err) => panic!("Failed to run launcher: {:?}", err),
    }
}
