use std::process;

use clap::Parser;

mod cairo_vm;
mod cli;
mod custom_rand;
mod fuzzer;
mod json;
mod mutator;

use cli::args::Opt;
use cli::config::Config;
use fuzzer::corpus::CrashFile;
use fuzzer::corpus::InputFile;
use fuzzer::fuzzer::Fuzzer;

use log::error;
fn main() {
    // get cli args
    let opt = Opt::parse();
    // create config file
    let config = match opt.config {
        // config file provided
        Some(config_file) => Config::load_config(&config_file),
        None => {
            if opt.contract.len() == 0 {
                error!("Fuzzer needs a contract path using --contract");
                process::exit(1);
            }
            if opt.function.len() == 0 {
                error!("Fuzzer needs a function name to fuzz using --function");
                process::exit(1);
            }

            Config {
                contract_file: opt.contract,
                function_name: opt.function,
                input_file: opt.inputfile,
                crash_file: opt.crashfile,
                cores: opt.cores,
                logs: opt.logs,
                seed: opt.seed,
                run_time: opt.run_time,
                replay: opt.replay,
                minimizer: opt.minimizer,
            }
        }
    };

    // create the fuzzer
    let mut fuzzer = Fuzzer::new(&config);

    // replay, minimizer mode
    // TODO - also when input file are provided, need to be replayed before fuzzing?  || (!&config.input_file.is_empty())
    if opt.replay || opt.minimizer {
        fuzzer.replay();
    // launch fuzzing
    } else {
        fuzzer.fuzz();
    }
}
