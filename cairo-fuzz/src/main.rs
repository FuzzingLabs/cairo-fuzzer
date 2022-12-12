use clap::Parser;
use json::json_parser::parse_json;
use json::json_parser::Function;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
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

#[derive(Debug)]

pub struct FuzzingData {
    contents: String,
    function: Function,
    seed: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCorpus {
    name: String,
    args: Vec<String>,
    inputs: Vec<Vec<u8>>,
    crash: Vec<Vec<u8>>,
}

pub fn cairo_fuzz(
    cores: i32,
    contract: &str,
    function_name: String,
    seed: Option<u64>,
    logs: bool,
) {
    // Global statistics
    let stats = Arc::new(Mutex::new(Statistics::default()));
    let mut log: Option<File> = None;
    // Open a log file
    if logs {
        log = Some(File::create("fuzz_stats.txt").unwrap());
    }
    // Save the current time
    let start_time = Instant::now();

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let seed = match seed {
        Some(val) => val,
        None => since_the_epoch.as_millis() as u64,
    };
    println!("Fuzzing SEED => {}", seed);
    let contents =
        fs::read_to_string(&contract.to_string()).expect("Should have been able to read the file");
    let function = match parse_json(&contents, &function_name.to_string()) {
        Some(func) => func,
        None => {
            println!("Could not find the function {}", function_name);
            return;
        }
    };
    let fuzzing_data = Arc::new(FuzzingData {
        contents: contents,
        function: function,
        seed: seed,
    });

    let types_args = fuzzing_data.function.type_args.clone();
    let func_corpus = Arc::new(Mutex::new(FunctionCorpus {
        name: function_name,
        args: types_args,
        inputs: Vec::<Vec<u8>>::new(),
        crash: Vec::<Vec<u8>>::new(),
    }));
    for i in 0..cores {
        // Spawn threads
        let stats = stats.clone();
        let fuzzing_data_clone = fuzzing_data.clone();
        let function_corpus = func_corpus.clone();
        let _ = std::thread::spawn(move || {
            worker(stats, function_corpus, i, fuzzing_data_clone);
        });
        println!("Thread {} Spawned", i);
    }

    loop {
        std::thread::sleep(Duration::from_millis(1000));

        // Get access to the global stats
        let stats = stats.lock().unwrap();

        let uptime = (Instant::now() - start_time).as_secs_f64();
        let fuzz_case = stats.fuzz_cases;
        print!(
            "{:12.2} uptime | {:9} fuzz cases | {:12.2} fcps | \
                    {:6} coverage | {:6} inputs | {:6} crashes [{:6} unique]\n",
            uptime,
            fuzz_case,
            fuzz_case as f64 / uptime,
            stats.coverage_db.len(),
            stats.input_db.len(),
            stats.crashes,
            stats.crash_db.len()
        );
        if let Some(ref mut file) = log {
            write!(
                file,
                "{:12.0} {:7} {:8} {:5} {:6} {:6}\n",
                uptime,
                fuzz_case,
                stats.coverage_db.len(),
                stats.input_db.len(),
                stats.crashes,
                stats.crash_db.len()
            )
            .unwrap();
            file.flush().unwrap();
        }
    }
}

pub fn cairo_replay(cores: i32, contract: &str, function_name: String, seed: Option<u64>) {
    let stats = Arc::new(Mutex::new(Statistics::default()));
    // Save the current time
    let start_time = Instant::now();
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let seed = match seed {
        Some(val) => val,
        None => since_the_epoch.as_millis() as u64,
    };
    println!("Fuzzing SEED => {}", seed);
    let contents =
        fs::read_to_string(&contract.to_string()).expect("Should have been able to read the file");
    let function = match parse_json(&contents, &function_name.to_string()) {
        Some(func) => func,
        None => {
            println!("Could not find the function {}", function_name);
            return;
        }
    };
    let fuzzing_data = Arc::new(FuzzingData {
        contents: contents,
        function: function,
        seed: seed,
    });
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
        let stats = stats.clone();
        let fuzzing_data_clone = fuzzing_data.clone();
        let chunk = chunks[i].clone();
        let _ = std::thread::spawn(move || {
            replay(stats, i, fuzzing_data_clone, &chunk);
        });
        println!("Thread {} Spawned", i);
    }

    loop {
        std::thread::sleep(Duration::from_millis(1000));

        // Get access to the global stats
        let stats = stats.lock().unwrap();
        println!("finished : {}", stats.finished);
        let uptime = (Instant::now() - start_time).as_secs_f64();
        let fuzz_case = stats.fuzz_cases;
        print!(
            "{:12.2} uptime | {:9} fuzz cases | {:12.2} fcps | \
                    {:6} coverage | {:6} inputs | {:6} crashes [{:6} unique]\n",
            uptime,
            fuzz_case,
            fuzz_case as f64 / uptime,
            stats.coverage_db.len(),
            stats.input_db.len(),
            stats.crashes,
            stats.crash_db.len()
        );
        if stats.finished as usize == chunks.len() {
            println!("STOP");
            break;
        }
    }
}

pub fn cairo_minimizer(contract: &str, function_name: String) {
    let contents =
        fs::read_to_string(&contract.to_string()).expect("Should have been able to read the file");
    let function = match parse_json(&contents, &function_name.to_string()) {
        Some(func) => func,
        None => {
            println!("Could not find the function {}", function_name);
            return;
        }
    };
    let fuzzing_data = FuzzingData {
        contents: contents,
        function: function,
        seed: 0,
    };
    let stats = Statistics::default();
    minimizer(stats, fuzzing_data, "./inputs".to_string());
}

fn main() {
    let opt = Opt::parse();
    let contract = opt
        .contract
        .to_str()
        .expect("Fuzzer needs path to contract");
    if opt.replay {
        cairo_replay(opt.cores, contract, opt.function.clone(), opt.seed)
    } else {
        cairo_fuzz(
            opt.cores,
            contract,
            opt.function.clone(),
            opt.seed,
            opt.logs,
        );
        if !opt.minimizer {
            cairo_fuzz(
                opt.cores,
                contract,
                opt.function.clone(),
                opt.seed,
                opt.logs,
            );
        } else {
            cairo_minimizer(contract, opt.function);
        }
    }
}