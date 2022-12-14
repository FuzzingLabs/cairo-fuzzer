use serde::{Deserialize, Serialize};
use std::path::Path;
use serde_json::Value;
use std::fs;
use crate::cairo_vm::cairo_types::CairoTypes;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputCorpus {
    pub name: String,
    pub args: Vec<String>,
    pub inputs: Vec<Vec<CairoTypes>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrashCorpus {
    pub name: String,
    pub args: Vec<String>,
    pub crashes: Vec<Vec<CairoTypes>>,
}

/// Function to load the previous corpus if it exists

pub fn load_corpus(inputs_corpus: &mut InputCorpus) {
    if Path::new(&format!("inputs_corpus/{}.json", inputs_corpus.name)).is_file() {
        let contents = fs::read_to_string(&format!("inputs_corpus/{}.json", inputs_corpus.name)).expect("Should have been able to read the file");
        let data: Value = serde_json::from_str(&contents).expect("JSON was not well-formatted");
        // TODO : NEED TO VERIFY IF THE ARGS IN THE JSON ARE THE SAME AS THE CAIRO ARTIFACT

        // Load old inputs to prevent overwriting and to use it as a dictionary for the mutator
        let inputs:Vec<Vec<CairoTypes>> = data["inputs"].as_array().unwrap().iter().map(|input_array| input_array.as_array().unwrap().iter().map(|input| input.as_u64().unwrap() as u8).collect()).collect();
        inputs_corpus.inputs.extend(inputs);
    }
}

/// 1st case - replay multiple inputs (.json) or multiple jsons with inputs
/// 2nd case - replay multiple crashes (.json) or multiple json with crashes
/// 3rd case - replay single input or crash (vector with data)

pub fn load_crashes(filename: String, crashes_corpus: &mut CrashCorpus) {
    unimplemented!("todo");
}