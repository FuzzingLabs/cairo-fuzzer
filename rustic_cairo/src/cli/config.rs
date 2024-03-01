use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

/// Config struct to use instead of command line
#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Config {
    pub workspace: String,
    pub contract_file: String,
    pub casm_file: String,
    pub target_function: String,
    pub statefull: bool,
    pub input_file: String,
    pub crash_file: String,
    pub corpus_dir: String,
    pub crashes_dir: String,
    pub dict: String,
    pub cores: i32,
    pub logs: bool,
    pub seed: Option<u64>,
    pub run_time: Option<u64>,
    pub replay: bool,
    pub minimizer: bool,
    pub proptesting: bool,
    pub iter: i64,
}

impl Config {
    /// Create a Config using the provided config file
    pub fn load_config(config_file: &String) -> Self {
        let config_string = fs::read_to_string(config_file).expect("Unable to read config file");
        return serde_json::from_str(&config_string).expect("Could not parse json config file");
    }
}
