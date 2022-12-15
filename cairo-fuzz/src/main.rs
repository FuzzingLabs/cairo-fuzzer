use clap::Parser;

mod cairo_vm;
mod cli;
mod custom_rand;
mod fuzzer;
mod json;
mod mutator;

use cli::args::Opt;
use fuzzer::corpus::CrashCorpus;
use fuzzer::corpus::InputCorpus;
use fuzzer::fuzzer::init_fuzzer;

fn main() {
    let opt = Opt::parse();
    let contract_file = opt
        .contract
        .to_str()
        .expect("Fuzzer needs path to contract")
        .to_string();
    let input_file = opt.inputfile.to_string();
    let crash_file = opt.crashfile.to_string();
    let mut fuzzer = init_fuzzer(
        opt.cores,
        opt.logs,
        opt.seed,
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
