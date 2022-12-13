use clap::Parser;
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
mod minimizer;
mod mutator;
mod replay;

use cli::args::Opt;
use fuzzer::stats::*;
use fuzzer::worker::worker;
use minimizer::minimizer::minimizer;
use replay::replay::replay;
use fuzzer::corpus::InputCorpus;
use fuzzer::corpus::CrashCorpus;
use fuzzer::corpus::load_corpus;
use fuzzer::stats::print_stats;

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
pub fn init_fuzzing_data(logs: bool, seed: Option<u64>, contract: String, function_name: String) -> FuzzingData {
    let start_time =  Instant::now();
    let stats = Arc::new(Mutex::new(Statistics::default()));
    let seed = match seed {
        Some(val) => val,
        None => SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
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
    let fuzzing_data =FuzzingData {
        stats: stats,
        logs: logs,
        contents: contents,
        function: function,
        start_time: start_time,
        seed: seed,
    };
    return fuzzing_data;
}

pub fn cairo_fuzz(
    cores: i32,
    contract: String,
    function_name: String,
    seed: Option<u64>,
    logs: bool,
) {
    // Set fuzzing data with the contents of the json artifact, the function data and the seed
    let fuzzing_data = Arc::new(init_fuzzing_data(logs, seed, contract, function_name.clone()));

    // Setup input corpus and crash corpus
    let mut inputs = InputCorpus {
        name: function_name.clone(),
        args: fuzzing_data.function.type_args.clone(),
        inputs: Vec::<Vec<u8>>::new(),
    };
    let crashes = CrashCorpus {
        name: function_name.clone(),
        args: fuzzing_data.function.type_args.clone(),
        crashes: Vec::<Vec<u8>>::new(),
    };

    // Load old corpus if exists
    load_corpus(&mut inputs);

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

/* pub fn cairo_replay(cores: i32, contract: String, function_name: String) {
    let fuzzing_data = Arc::new(init_fuzzing_data(false, None, contract, function_name));

    let files: Vec<String> = fs::read_dir("./inputs".to_string())
        .unwrap()
        .map(|file| file.unwrap().path().to_str().unwrap().to_string())
        .collect();
    // Split the files into chunks
    let chunk_size = files.len() / (files.len() / (cores as usize));
    let mut chunks = Vec::new();
    for chunk in files.chunks(chunk_size) {
        chunks.push(chunk.to_vec());
    }
    println!("Total files => {}", files.len());
    for i in 0..chunks.len() {
        // Spawn threads
        let fuzzing_data_clone = fuzzing_data.clone();
        let chunk = chunks[i].clone();
        let _ = std::thread::spawn(move || {
            replay( i, fuzzing_data_clone, &chunk);
        });
        println!("Thread {} Spawned", i);
    }

    print_stats(fuzzing_data);
} */


fn main() {
    let opt = Opt::parse();
    let contract = opt
        .contract
        .to_str()
        .expect("Fuzzer needs path to contract");
   /*  if opt.replay {
        cairo_replay(opt.cores, contract.to_string(), opt.function.clone());
    } else { */
        cairo_fuzz(
            opt.cores,
            contract.to_string(),
            opt.function.clone(),
            opt.seed,
            opt.logs,
        );
/*         if !opt.minimizer {
            cairo_fuzz(
                opt.cores,
                contract.to_string(),
                opt.function.clone(),
                opt.seed,
                opt.logs,
            );
        }
    } */
}
