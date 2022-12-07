use clap::Parser;
use json::json_parser::parse_json;
use json::json_parser::Function;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

mod cairo_vm;
mod cli;
mod custom_rand;
mod fuzzer;
mod json;
// JSON parsing
use cli::args::Opt;
use fuzzer::stats::*;
use fuzzer::worker::worker;

#[derive(Debug)]

pub struct FuzzingData {
    contents: String,
    function: Function,
    seed: Option<u64>,
}

fn main() {
    let opt = Opt::parse();
    let cores = opt.cores;
    let contract = opt
        .contract
        .to_str()
        .expect("Fuzzer needs path to contract");
    let seed = opt.seed;
    let function_name = opt.function;
    // Global statistics
    let stats = Arc::new(Mutex::new(Statistics::default()));

    // Open a log file
    let mut log = File::create("fuzz_stats.txt").unwrap();

    // Save the current time
    let start_time = Instant::now();

    // TODO - get those info from main
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
    for i in 0..cores {
        // Spawn threads
        let stats = stats.clone();
        let fuzzing_data_clone = fuzzing_data.clone();
        let _ = std::thread::spawn(move || {
            worker(stats, i, fuzzing_data_clone);
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
            "{:12.2} uptime | {:7} fuzz cases | {} fcps | \
                {:8} coverage | {:5} inputs | {:6} crashes [{:6} unique]\n",
            uptime,
            fuzz_case,
            fuzz_case as f64 / uptime,
            stats.coverage_db.len(),
            stats.input_db.len(),
            stats.crashes,
            stats.crash_db.len()
        );

        write!(
            log,
            "{:12.0} {:7} {:8} {:5} {:6} {:6}\n",
            uptime,
            fuzz_case,
            stats.coverage_db.len(),
            stats.input_db.len(),
            stats.crashes,
            stats.crash_db.len()
        )
        .unwrap();
        log.flush().unwrap();
    }
}
