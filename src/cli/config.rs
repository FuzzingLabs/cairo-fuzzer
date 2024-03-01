use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

/// Config struct to use instead of command line
#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Config {
    pub contract_file: String,
    pub casm_file: String,
    pub target_function: String,
    pub statefull: bool,
    pub corpus_dir: String,
    pub crashes_dir: String,
    pub cores: i32,
    pub seed: Option<u64>,
    pub replay: bool,
    pub proptesting: bool,
    pub diff_fuzz: bool,
}

impl Config {
    /// Create a Config using the provided config file
    pub fn load_config(config_file: &String) -> Self {
        let config_string = fs::read_to_string(config_file).expect("Unable to read config file");
        return serde_json::from_str(&config_string).expect("Could not parse json config file");
    }
}
