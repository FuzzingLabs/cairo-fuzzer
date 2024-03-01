use std::{fs, process};

use clap::Parser;

use crate::fuzzer::fuzzer::Fuzzer;
use cli::args::Opt;
use cli::config::Config;
use fuzzer::fuzzer_utils::replay;
use log::error;

mod cli;
mod fuzzer;
mod json_helper;
mod mutator;
mod runner;
mod ui;
mod worker;

fn main() {
    let args = Opt::parse();
    if args.analyze {
        let contents = fs::read_to_string(&args.contract).unwrap();
        json_helper::json_parser::analyze_json(&contents);
        return;
    }
    let config = match args.config {
        // config file provided
        Some(config_file) => Config::load_config(&config_file),
        None => {
            if args.contract.len() == 0 && args.proptesting == false {
                error!("Fuzzer needs a contract path using --contract");
                process::exit(1);
            }
            if args.target_function.len() == 0 && args.proptesting == false {
                error!("Fuzzer needs a function name to fuzz using --function");
                process::exit(1);
            }

            Config {
                diff_fuzz: args.diff_fuzz,
                statefull: args.statefull,
                contract_file: args.contract,
                casm_file: args.casm,
                target_function: args.target_function,
                corpus_dir: args.corpus_dir,
                crashes_dir: args.crashes_dir,
                cores: args.cores,
                seed: args.seed,
                replay: args.replay,
                proptesting: args.proptesting,
            }
        }
    };
    if config.proptesting {
        let contents = fs::read_to_string(&config.contract_file).unwrap();
        let functions = json_helper::json_parser::get_proptesting_functions(&contents);
        for func in functions {
            let mut func_config = config.clone();
            func_config.target_function = func;
            let mut fuzzer = Fuzzer::new(func_config);
            fuzzer.run(config.proptesting);
        }
    } else if config.replay {
        replay(&config, config.crashes_dir.as_str());
    } else {
        // create the fuzzer
        let mut fuzzer = Fuzzer::new(config.clone());
        fuzzer.run(config.proptesting);
    }
}
