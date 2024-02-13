use std::{fs, process};

use clap::Parser;

use cli::config::Config;
use fuzzer::fuzzer_utils::replay;
use crate::fuzzer::fuzzer::Fuzzer;
use cli::args::Opt;
use log::error;

mod json_helper;
mod fuzzer;
mod mutator;
mod runner;
mod ui;
mod worker;
mod cli;

fn main() {
    let args = Opt::parse();
    if args.analyze {
        let contents = fs::read_to_string(&args.contract).unwrap();
        json_helper::json_parser::analyze_json(&contents);
        return;
    }
    let mut config = match args.config {
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
                workspace: args.workspace,
                contract_file: args.contract,
                casm_file: args.casm,
                target_function: args.target_function,
                input_file: args.inputfile,
                crash_file: args.crashfile,
                corpus_dir: args.corpus_dir,
                crashes_dir: args.crashes_dir,
                dict: args.dict,
                cores: args.cores,
                logs: args.logs,
                seed: args.seed,
                run_time: args.run_time,
                replay: args.replay,
                minimizer: args.minimizer,
                proptesting: args.proptesting,
                iter: args.iter,
            }
        }
    };
        if config.proptesting {
        let contents = fs::read_to_string(&config.contract_file).unwrap();
        println!("\t\t\t\t\t\t\tSearching for Fuzzing functions ...");
        let functions = json_helper::json_parser::get_proptesting_functions(&contents);
        if functions.len() == 0 {
            println!("\t\t\t\t\t\t\t!! No Fuzzing functions found !!");
            return;
        }
        todo!()
/*         for func in functions {
            println!("\n\t\t\t\t\t\t\tFunction found => {}", &func);
            config.target_function = func;
            let mut fuzzer = Fuzzer::new(config);
            println!(
                "\t\t\t\t\t\t\t=== {} === is now running for {} iterations",
                config.target_function, config.iter
            );
            fuzzer.run();
        } */
    } else {
        // create the fuzzer
        let mut fuzzer = Fuzzer::new(config);

        // replay, minimizer mode
        if args.replay || args.minimizer {
            //replay(&config, args.crashes_dir.as_str());
            todo!()
        // launch fuzzing
        } else {
            fuzzer.run();
         }
    }
}
