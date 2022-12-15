use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub cores: i32,
    pub logs: bool,
    pub seed: Option<u64>,
    pub replay: bool,
    pub minimizer: bool,
    pub contract_file: String,
    pub function_name: String,
    pub input_file: String,
    pub crash_file: String,
}

pub fn load_config(config_file: &String) -> Config {
    let config_string = fs::read_to_string(config_file).expect("Unable to read config file");
    let config: Config =
        serde_json::from_str(&config_string).expect("Could not parse json config file");

    return config;
}
