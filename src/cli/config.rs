//! This module contains the config struct and it's implementation. If cairo-fuzzer is used as lib, a config file or a config struct can be used instead of the CLI args.
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

/// Config struct to use instead of command line
#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Config {
    pub workspace: String,
    pub contract_file: String,
    pub function_name: String,
    pub input_file: String,
    pub crash_file: String,
    pub input_folder: String,
    pub crash_folder: String,
    pub cores: i32,
    pub logs: bool,
    pub stdout: bool,
    pub seed: Option<u64>,
    pub run_time: Option<u64>,
    pub replay: bool,
    pub minimizer: bool,
    pub abi_path: Option<String>,
    pub devnet_host: Option<String>,
    pub devnet_port: Option<String>,
    pub cairo: bool,
    pub starknet: bool,
}

impl Config {
    /// Create a Config using the provided config file
    pub fn load_config(config_file: &String) -> Self {
        let config_string = fs::read_to_string(config_file).expect("Unable to read config file");
        return serde_json::from_str(&config_string).expect("Could not parse json config file");
    }
}
