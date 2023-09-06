use std::{fs, process};

use clap::Parser;

mod cli;
mod custom_rand;
mod fuzzer;
mod json;
mod mutator;
mod runner;

use cli::args::Opt;
use cli::config::Config;
use fuzzer::fuzzer::Fuzzer;

use log::error;

fn main() {
    /*let header = r###"
                                _______  _______ _________ _______  _______         _______           _______  _______  _______  _______
                                (  ____ \(  ___  )\__   __/(  ____ )(  ___  )       (  ____ \|\     /|/ ___   )/ ___   )(  ____ \(  ____ )
                                | (    \/| (   ) |   ) (   | (    )|| (   ) |       | (    \/| )   ( |\/   )  |\/   )  || (    \/| (    )|
                                | |      | (___) |   | |   | (____)|| |   | | _____ | (__    | |   | |    /   )    /   )| (__    | (____)|
                                | |      |  ___  |   | |   |     __)| |   | |(_____)|  __)   | |   | |   /   /    /   / |  __)   |     __)
                                | |      | (   ) |   | |   | (\ (   | |   | |       | (      | |   | |  /   /    /   /  | (      | (\ (
                                | (____/\| )   ( |___) (___| ) \ \__| (___) |       | )      | (___) | /   (_/\ /   (_/\| (____/\| ) \ \__
                                (_______/|/     \|\_______/|/   \__/(_______)       |/       (_______)(_______/(_______/(_______/|/   \__/"###;
    */
    // get cli args
    let opt = Opt::parse();
    // create config file
    let mut config = match opt.config {
        // config file provided
        Some(config_file) => Config::load_config(&config_file),
        None => {
            if opt.contract.len() == 0 && opt.proptesting == false {
                error!("Fuzzer needs a contract path using --contract");
                process::exit(1);
            }
            if opt.function.len() == 0 && opt.proptesting == false {
                error!("Fuzzer needs a function name to fuzz using --function");
                process::exit(1);
            }

            Config {
                workspace: opt.workspace,
                contract_file: opt.contract,
                casm_file: opt.casm,
                function_name: opt.function,
                input_file: opt.inputfile,
                crash_file: opt.crashfile,
                input_folder: opt.inputfolder,
                crash_folder: opt.crashfolder,
                dict: opt.dict,
                cores: opt.cores,
                logs: opt.logs,
                seed: opt.seed,
                run_time: opt.run_time,
                replay: opt.replay,
                minimizer: opt.minimizer,
                proptesting: opt.proptesting,
                iter: opt.iter,
            }
        }
    };
    if config.proptesting {
        let contents = fs::read_to_string(&config.contract_file).unwrap();
        println!("\t\t\t\t\t\t\tSearching for Fuzzing functions ...");
        let functions = json::json_parser::get_proptesting_functions(&contents);
        if functions.len() == 0 {
            println!("\t\t\t\t\t\t\t!! No Fuzzing functions found !!");
            return;
        }
        for func in functions {
            println!("\n\t\t\t\t\t\t\tFunction found => {}", &func);
            config.function_name = func;
            let mut fuzzer = Fuzzer::new(&config);
            println!(
                "\t\t\t\t\t\t\t=== {} === is now running for {} iterations",
                config.function_name, config.iter
            );
            fuzzer.fuzz();
        }
    } else {
        // create the fuzzer
        let mut fuzzer = Fuzzer::new(&config);

        // replay, minimizer mode
        if opt.replay || opt.minimizer {
            fuzzer.replay();
        // launch fuzzing
        } else {
            fuzzer.fuzz();
        }
    }
}
