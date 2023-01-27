use std::process;

use clap::Parser;

mod cairo_vm;
mod cli;
mod custom_rand;
mod fuzzer;
mod json;
mod mutator;
mod starknet_helper;

use cli::args::Opt;
use cli::config::Config;
use fuzzer::fuzzer::Fuzzer;

use log::error;
fn main() {
    let header = r###"
                 _______  _______ _________ _______  _______         _______           _______  _______  _______  _______ 
                (  ____ \(  ___  )\__   __/(  ____ )(  ___  )       (  ____ \|\     /|/ ___   )/ ___   )(  ____ \(  ____ )
                | (    \/| (   ) |   ) (   | (    )|| (   ) |       | (    \/| )   ( |\/   )  |\/   )  || (    \/| (    )|
                | |      | (___) |   | |   | (____)|| |   | | _____ | (__    | |   | |    /   )    /   )| (__    | (____)|
                | |      |  ___  |   | |   |     __)| |   | |(_____)|  __)   | |   | |   /   /    /   / |  __)   |     __)
                | |      | (   ) |   | |   | (\ (   | |   | |       | (      | |   | |  /   /    /   /  | (      | (\ (   
                | (____/\| )   ( |___) (___| ) \ \__| (___) |       | )      | (___) | /   (_/\ /   (_/\| (____/\| ) \ \__
                (_______/|/     \|\_______/|/   \__/(_______)       |/       (_______)(_______/(_______/(_______/|/   \__/"###;
    println!("\t=========================================================================================================================");
    println!("{}", header);
    println!("\n\t=========================================================================================================================");
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
                abi_path: None,
                devnet_host: None,
                devnet_port: None,
                workspace: opt.workspace,
                contract_file: opt.contract,
                function_name: opt.function,
                input_file: opt.inputfile,
                crash_file: opt.crashfile,
                input_folder: opt.inputfolder,
                crash_folder: opt.crashfolder,
                cores: opt.cores,
                logs: opt.logs,
                stdout: opt.stdout,
                seed: opt.seed,
                run_time: opt.run_time,
                replay: opt.replay,
                minimizer: opt.minimizer,
                cairo: true,
                starknet: false,
            }
        }
    };

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
