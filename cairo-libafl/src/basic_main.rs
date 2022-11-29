use std::path::PathBuf;
#[cfg(windows)]
use std::ptr::write_volatile;

#[cfg(feature = "tui")]
use libafl::monitors::tui::TuiMonitor;
#[cfg(not(feature = "tui"))]
use libafl::monitors::SimpleMonitor;
use libafl::{
    bolts::{current_nanos, rands::StdRand, tuples::tuple_list, AsSlice},
    corpus::{InMemoryCorpus, OnDiskCorpus},
    events::SimpleEventManager,
    executors::{inprocess::InProcessExecutor, ExitKind},
    feedbacks::{CrashFeedback, MaxMapFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    generators::RandPrintablesGenerator,
    inputs::{BytesInput, HasTargetBytes},
    mutators::scheduled::{havoc_mutations, StdScheduledMutator},
    observers::StdMapObserver,
    schedulers::QueueScheduler,
    stages::mutational::StdMutationalStage,
    state::StdState,
};
use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::types::program::Program;
use cairo_rs::types::relocatable::MaybeRelocatable;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;
use num_bigint::BigInt;
use num_bigint::Sign;
use std::any::Any;
use std::env;
use std::fs;
use std::path::Path;
mod parse_json;
use crate::parse_json::parse_json;
mod utils;
use std::thread;

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

#[allow(clippy::similar_names)]
pub fn main() {
    let functions = parse_json(&"json/vuln.json".to_string());
    let contents = fs::read_to_string(&"json/vuln.json".to_string())
        .expect("Should have been able to read the file");
    // The closure that we want to fuzz
    let mut harness = |input: &BytesInput| {
        for function in functions.clone() {
            signals_set(1); // set SIGNALS[1]
            runner(&contents, function.name, function.num_args, 2);
            signals_set(2); // set SIGNALS[2]
        }
        ExitKind::Ok
    };

    // Create an observation channel using the signals map
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

    // The Monitor trait define how the fuzzer stats are displayed to the user
    #[cfg(not(feature = "tui"))]
    let mon = SimpleMonitor::new(|s| println!("{}", s));
    #[cfg(feature = "tui")]
    let mon = TuiMonitor::new(String::from("Baby Fuzzer"), false);

    // The event manager handle the various events generated during the fuzzing loop
    // such as the notification of the addition of a new item to the corpus
    let mut mgr = SimpleEventManager::new(mon);

    // A queue policy to get testcasess from the corpus
    let scheduler = QueueScheduler::new();

    // A fuzzer with feedbacks and a corpus scheduler
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    // Create the executor for an in-process function with just one observer
    let mut executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(observer),
        &mut fuzzer,
        &mut state,
        &mut mgr,
    )
    .expect("Failed to create the Executor");

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
}