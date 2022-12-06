use cairo_rs::types::relocatable::Relocatable;
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
use input_generator::generator::MyRandPrintablesGenerator;
use libafl::prelude::HavocMutationsType;
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
    mutators::scheduled::{havoc_mutations, StdScheduledMutator},
    observers::StdMapObserver,
    schedulers::QueueScheduler,
    stages::mutational::StdMutationalStage,
    state::StdState,
    Error,
};
use libafl_targets::{CmpLogObserver, CMPLOG_MAP, EDGES_MAP, MAX_EDGES_NUM};
use std::{fs, path::PathBuf};
use utils::parse_json::parse_json;
/// Coverage map with explicit assignments due to the lack of instrumentation
//static mut SIGNALS: [u8; 100] = [0; 100];
//#[derive(SliceIndex)]
//static mut SIGNALS: [(usize, usize); 100] = [(0, 0); 100];

/// Assign a signal to the signals map
//fn signals_set(fp_off: usize, pc_off: usize) {
//    unsafe { SIGNALS[(fp_off, pc_off)] = 1 };
//}

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
        let edges = unsafe { &mut EDGES_MAP[0..MAX_EDGES_NUM] };
        let edges_observer = StdMapObserver::new("edges", edges);

        // Create an observation channel to keep track of the execution time
        let time_observer = TimeObserver::new("time");

        let cmplog = unsafe { &mut CMPLOG_MAP };
        let cmplog_observer = CmpLogObserver::new("cmplog", cmplog, true);

        //let observer = ListObserver::new("cov", unsafe { &mut COVERAGE });

        // Feedback to rate the interestingness of an input
        //let mut feedback = ListFeedback::new_with_observer(&observer);

        let mut feedback = feedback_or!(
            // New maximization map feedback linked to the edges observer and the feedback state
            MaxMapFeedback::new_tracking(&edges_observer, true, false),
            // Time feedback, this one does not need a feedback state
            TimeFeedback::new_with_observer(&time_observer)
        );

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
        let mut trace_feedback = Vec::<(Relocatable, Relocatable)>::new();
        let mut index: usize = 1;
        // The wrapped harness function
        let mut harness = |input: &BytesInput| {
            let target = input.target_bytes();
            let buf = target.as_slice();
            if !buf.is_empty() && buf.len() == 11 {
                match runner(&contents, function.name.clone(), input) {
                    Ok(trace) => {
                        for i in trace.unwrap() {
                            //if !trace_feedback.contains(&i) {
                            //    trace_feedback.push(i);
                            //}

                            /* how to use edges/edges map ?? */

                            //signals_set(i.0.offset, i.1.offset);
                            //    index += 1;
                            // }
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
            tuple_list!(edges_observer, time_observer),
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
