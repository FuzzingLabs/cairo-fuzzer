use std::{
    collections::HashSet,
    fs::{self, File},
    io::Write,
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use starknet_rs::CasmContractClass;

use super::{coverage::Coverage, crash::Crash};
use crate::{
    cli::config::Config, json_helper::json_parser::get_function_from_json, runner::runner::Runner,
};

pub fn write_crashfile(path: &str, crash: Crash) {
    if let Err(err) = fs::create_dir_all(path) {
        panic!("Could not create crashes directory: {}", err);
    }
    let d = SystemTime::now();
    // Create DateTime from SystemTime
    let datetime = DateTime::<Utc>::from(d);
    // Formats the combined date and time with the specified format string.
    let timestamp_str = datetime.format("%Y-%m-%d--%H:%M:%S").to_string();
    let mut file = File::create(format!(
        "{}/{}-{}.json",
        path, timestamp_str, crash.target_function
    ))
    .unwrap();
    file.write_all(serde_json::to_string(&crash).unwrap().as_bytes())
        .unwrap();
}

pub fn write_corpusfile(path: &str, cov: &Coverage) {
    if let Err(err) = fs::create_dir_all(path) {
        panic!("Could not create crashes directory: {}", err);
    }
    let d = SystemTime::now();
    // Create DateTime from SystemTime
    let datetime = DateTime::<Utc>::from(d);
    // Formats the combined date and time with the specified format string.
    let timestamp_str = datetime.format("%Y-%m-%d--%H:%M:%S").to_string();
    let mut file = File::create(format!("{}/{}.json", path, timestamp_str)).unwrap();
    file.write_all(serde_json::to_string(&cov).unwrap().as_bytes())
        .unwrap();
}

pub fn replay(config: &Config, crashfile_path: &str) {
    let data = fs::read_to_string(crashfile_path).expect("Could not read crash file !");
    let crash: Crash = serde_json::from_str(&data).expect("Could not load crash file !");
    let contents =
        fs::read_to_string(&config.contract_file).expect("Should have been able to read the file");
    let casm_content = fs::read_to_string(&config.casm_file).expect("Could not read casm file");
    let contract_class: CasmContractClass =
        serde_json::from_str(&casm_content).expect("could not get contractclass");
    let target_function = match get_function_from_json(&contents, &config.target_function) {
        Some(func) => func,
        None => {
            eprintln!("Error: Could not parse json file");
            return;
        }
    };
    let mut runner = crate::runner::starknet_runner::RunnerStarknet::new(
        &contract_class,
        target_function.clone(),
    );
    match runner.execute(crash.inputs) {
        Ok(_) => unreachable!(),
        Err(e) => println!("{:?}", e.1),
    }
}

pub fn load_corpus(path: &str) -> Result<HashSet<Coverage>, String> {
    let mut set = HashSet::new();
    if let Ok(paths) = fs::read_dir(path) {
        for file in paths {
            let data = fs::read_to_string(file.unwrap().path().display().to_string())
                .expect("Could not read corpus file !");
            let coverage = serde_json::from_str(&data).unwrap();
            set.insert(coverage);
        }
        return Ok(set);
    }
    Err("Could not read corpus directory !".to_string())
}

pub fn load_crashes(path: &str) -> Result<HashSet<Crash>, String> {
    let mut set = HashSet::new();
    if let Ok(paths) = fs::read_dir(path) {
        for file in paths {
            let data = fs::read_to_string(file.unwrap().path().display().to_string())
                .expect("Could not read crash file !");
            let crash = serde_json::from_str(&data).unwrap();
            set.insert(crash);
        }
        return Ok(set);
    }
    Err("Could not read crash directory !".to_string())
}
