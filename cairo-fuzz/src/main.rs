use std::process;

use clap::Parser;

mod cairo_vm;
mod cli;
mod custom_rand;
mod fuzzer;
mod json;
mod mutator;

use cli::args::Opt;
use cli::config::load_config;
use fuzzer::corpus::CrashCorpus;
use fuzzer::corpus::InputCorpus;
use fuzzer::fuzzer::init_fuzzer;
use fuzzer::fuzzer::init_fuzzer_from_config;
use log::error;
fn main() {
    let opt = Opt::parse();
    if let Some(config_file) = opt.config {
        let config = load_config(&config_file);
        let mut fuzzer = init_fuzzer_from_config(config.clone());
        if config.replay || config.minimizer {
            fuzzer.replay();
        } else {
            fuzzer.fuzz();
        }
    } else {
        if opt.contract.len() == 0 {
            error!("Fuzzer needs a contract path using --contract");
            process::exit(1);
        }
        if opt.function.len() == 0 {
            error!("Fuzzer needs a function name to fuzz using --function");
            process::exit(1);
        }
        let contract_file = opt.contract;
        let input_file = opt.inputfile;
        let crash_file = opt.crashfile;
        let mut fuzzer = init_fuzzer(
            opt.cores,
            opt.logs,
            opt.seed,
            opt.timeout,
            opt.replay,
            opt.minimizer,
            &contract_file,
            &opt.function,
            &input_file,
            &crash_file,
        );
        if opt.replay || opt.minimizer {
            fuzzer.replay();
        } else {
            fuzzer.fuzz();
        }
    }
}
