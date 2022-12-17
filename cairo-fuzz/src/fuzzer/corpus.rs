use crate::cairo_vm::cairo_types::Felt;
use serde::{Deserialize, Serialize};
use std::fs::create_dir;
use std::fs::write;
use crate::json::json_parser::Function;
use std::path::Path;
use std::fs;
use serde_json::Value;

use std::collections::{HashSet};

#[derive(Debug, Clone)]
pub struct Workspace {
    workspace_folder: String,
    input_folder: String,
    crash_folder: String,
}

// TODO - improve and allow user to choose
impl Default for Workspace {
    fn default() -> Self {
        Workspace {
            workspace_folder: "ws".to_string(), // use contract name??
            input_folder: "inputs_corpus".to_string(),
            crash_folder: "crashes_corpus".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InputFile {
    pub path: String,
    pub name: String,
    pub args: Vec<String>,
    pub inputs: Vec<Vec<Felt>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CrashFile {
    pub path: String,
    pub name: String,
    pub args: Vec<String>,
    pub crashes: Vec<Vec<Felt>>,
}

/// TODO - Load all inputs files
fn load_inputs(folder_path: &String) -> Vec<InputFile> {
    // if Path::new(&filename).is_file() {
    unimplemented!();
}

/// TODO - Load all crashes files
fn load_crashes(folder_path: &String) -> Vec<CrashFile> {
    unimplemented!();
}

impl InputFile {

    pub fn new_from_function(function: &Function) -> Self {
        InputFile {
            path: format!("inputs_corpus/{}_inputs.json", function.name),
            name: function.name.clone(),
            args: function.type_args.clone(),
            inputs: Vec::<Vec<Felt>>::new(),
        }
    }

    /// Function to load the previous corpus if it exists
    pub fn load_from_file(filename: &String) -> Self {
        // Try to load the file
        let contents =
            fs::read_to_string(filename).expect("Should have been able to read the file");
        // Extract json data
        let data: Value = serde_json::from_str(&contents).expect("JSON was not well-formatted");
        // Load inputs
        let inputs: Vec<Vec<Felt>> = data["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|input_array| {
                input_array
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|input| input.as_u64().unwrap() as Felt)
                    .collect()
            })
            .collect();

        return InputFile {
            path: filename.clone(),
            name: data["name"].as_str().unwrap().to_string(),
            args: data["args"].as_array().unwrap().iter().map(|input_array| input_array.as_str().unwrap().to_string()).collect(),
            inputs: inputs,
        };
    }


    /// Function to dump the inputs corpus
    pub fn dump_json(&self) {
        let workspace = Workspace::default();
        let _ = create_dir(&workspace.crash_folder);
        let _ = create_dir(&workspace.input_folder);
        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");

        let mut inputs_ser = serde_json::Serializer::with_formatter(buf.clone(), formatter.clone());
        self.serialize(&mut inputs_ser).unwrap();
        write(
            &self.path,
            String::from_utf8(inputs_ser.into_inner()).unwrap(),
        )
        .expect("Failed to save input to disk");
    }
}

impl CrashFile {

    pub fn new_from_function(function: &Function) -> Self {
        CrashFile {
            path: format!("inputs_corpus/{}_inputs.json", function.name),
            name: function.name.clone(),
            args: function.type_args.clone(),
            crashes: Vec::<Vec<Felt>>::new(),
        }
    }

    /// Function to load a crashes corpus
    pub fn load_from_file(filename: &String) -> Self {

        // Try to load the file
        let contents =
            fs::read_to_string(filename).expect("Should have been able to read the file");
        // Extract json data
        let data: Value = serde_json::from_str(&contents).expect("JSON was not well-formatted");
        // Load old crashes to prevent overwriting and to use it as a dictionary for the mutator
        let crashes: Vec<Vec<Felt>> = data["crashes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|input_array| {
                input_array
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|input| input.as_u64().unwrap() as Felt)
                    .collect()
            })
            .collect();

        return CrashFile {
            path: filename.clone(),
            name: data["name"].as_str().unwrap().to_string(),
            args: data["args"].as_array().unwrap().iter().map(|input_array| input_array.as_str().unwrap().to_string()).collect(),
            crashes: crashes,
        };

    }

    /// Function to dump the crashes corpus
    pub fn dump_json(&self) {
        let workspace = Workspace::default();
        let _ = create_dir(&workspace.crash_folder);
        let _ = create_dir(&workspace.input_folder);
        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        // TODO - Change name of crashes files by adding the date and the time

        let mut crashes_ser = serde_json::Serializer::with_formatter(buf.clone(), formatter.clone());
        self.serialize(&mut crashes_ser).unwrap();
        write(
            &self.path,
            String::from_utf8(crashes_ser.into_inner()).unwrap(),
        )
        .expect("Failed to save input to disk");
    }
}
