use clap::Parser;
use fuzzer::corpus::load_crashes_corpus;
use fuzzer::corpus::load_inputs_corpus;
use json::json_parser::parse_json;
use json::json_parser::Function;
use std::fs;
use std::process;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

mod cairo_vm;
mod cli;
mod custom_rand;
mod fuzzer;
mod json;
mod mutator;

use cli::args::Opt;
use fuzzer::corpus::CrashCorpus;
use fuzzer::corpus::InputCorpus;
use fuzzer::fuzzing_worker::worker;
use fuzzer::replay_worker::replay;
use fuzzer::stats::print_stats;
use fuzzer::stats::*;

#[derive(Debug)]
pub struct FuzzingData {
    stats: Arc<Mutex<Statistics>>,
    logs: bool,
    contents: String,
    function: Function,
    start_time: Instant,
    seed: u64,
}

/// Init all the fuzzing data the fuzzer will need to send to the different workers
pub fn init_fuzzing_data(
    logs: bool,
    seed: Option<u64>,
    contract: String,
    function_name: String,
) -> FuzzingData {
    let start_time = Instant::now();
    let stats = Arc::new(Mutex::new(Statistics::default()));
    let seed = match seed {
        Some(val) => val,
        None => SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };
    println!("Fuzzing SEED => {}", seed);
    // Read json artifact and get its content
    let contents =
        fs::read_to_string(&contract.to_string()).expect("Should have been able to read the file");
    let function = match parse_json(&contents, &function_name.to_string()) {
        Some(func) => func,
        None => {
            process::exit(1);
        }
    };
    let fuzzing_data = FuzzingData {
        stats: stats,
        logs: logs,
        contents: contents,
        function: function,
        start_time: start_time,
        seed: seed,
    };
    return fuzzing_data;
}

/// Run the fuzzing worker
pub fn cairo_fuzz(
    cores: i32,
    contract: String,
    function_name: String,
    seed: Option<u64>,
    logs: bool,
    input_file: String,
    crash_file: String,
) {
    // Set fuzzing data with the contents of the json artifact, the function data and the seed
    let fuzzing_data = Arc::new(init_fuzzing_data(
        logs,
        seed,
        contract,
        function_name.clone(),
    ));

    // Setup input corpus and crash corpus
    let inputs = load_inputs_corpus(fuzzing_data.clone(), input_file);

    let crashes = load_crashes_corpus(fuzzing_data.clone(), crash_file);
    // Setup the mutex for the inputs corpus and crash corpus
    let inputs = Arc::new(Mutex::new(inputs));
    let crashes = Arc::new(Mutex::new(crashes));
    // Running all the threads
    for i in 0..cores {
        // Spawn threads
        let fuzzing_data_clone = fuzzing_data.clone();
        let inputs_corpus = inputs.clone();
        let crashes_corpus = crashes.clone();
        let _ = std::thread::spawn(move || {
            worker(inputs_corpus, crashes_corpus, i, fuzzing_data_clone);
        });
        println!("Thread {} Spawned", i);
    }

    // Call the stats printer
    print_stats(fuzzing_data);
}

pub fn cairo_replay(
    cores: i32,
    contract: String,
    function_name: String,
    input_file: String,
    crash_file: String,
    minimizer: bool,
) {
    let fuzzing_data = Arc::new(init_fuzzing_data(false, None, contract, function_name));
    let inputs = load_inputs_corpus(fuzzing_data.clone(), input_file);
    let crashes = load_crashes_corpus(fuzzing_data.clone(), crash_file);
    let corpus = if inputs.inputs.len() != 0 {
        inputs.inputs
    } else {
        crashes.crashes
    };
    // Split the files into chunks
    let chunk_size = corpus.len() / ((corpus.len() / (cores as usize)) + 1);
    let mut chunks = Vec::new();
    for chunk in corpus.chunks(chunk_size) {
        chunks.push(chunk.to_vec());
    }
    println!("Total inputs => {}", corpus.len());
    for i in 0..chunks.len() {
        // Spawn threads
        let fuzzing_data_clone = fuzzing_data.clone();
        let chunk = chunks[i].clone();
        let _ = std::thread::spawn(move || {
            replay(i, fuzzing_data_clone, chunk, minimizer);
        });
        println!("Thread {} Spawned", i);
    }

    print_stats(fuzzing_data);
}

fn main() {
    let opt = Opt::parse();
    let contract = opt
        .contract
        .to_str()
        .expect("Fuzzer needs path to contract");
    let input_file = opt.input_file.to_string();
    let crash_file = opt.crash_file.to_string();
    if opt.replay || opt.minimizer {
        cairo_replay(
            opt.cores,
            contract.to_string(),
            opt.function.clone(),
            input_file,
            crash_file,
            opt.minimizer,
        );
    } else {
        cairo_fuzz(
            opt.cores,
            contract.to_string(),
            opt.function.clone(),
            opt.seed,
            opt.logs,
            input_file,
            crash_file,
        );
    }
}
