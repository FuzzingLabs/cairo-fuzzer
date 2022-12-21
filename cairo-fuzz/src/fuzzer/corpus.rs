use crate::cairo_vm::cairo_types::Felt;
use crate::json::json_parser::Function;
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::fs::create_dir;
use std::fs::write;
use std::time::SystemTime;

/*
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
            workspace_folder: "seth_workspace".to_string(),
            input_folder: "inputs_corpus".to_string(),
            crash_folder: "crashes_corpus".to_string(),
        }
    }
} */

/* /// TODO - Load all inputs files
fn load_inputs(folder_path: &String) -> Vec<InputFile> {
    // if Path::new(&filename).is_file() {
    unimplemented!();
}

/// TODO - Load all crashes files
fn load_crashes(folder_path: &String) -> Vec<CrashFile> {
    unimplemented!();
} */

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InputFile {
    pub workspace: String,
    pub path: String,
    pub name: String,
    pub args: Vec<String>,
    pub inputs: Vec<Vec<Felt>>,
}

impl InputFile {
    pub fn new_from_function(function: &Function, workspace: &String) -> Self {
        let d = SystemTime::now();
        // Create DateTime from SystemTime
        let datetime = DateTime::<Utc>::from(d);
        // Formats the combined date and time with the specified format string.
        let timestamp_str = datetime.format("%Y-%m-%d--%H:%M:%S").to_string();
        InputFile {
            workspace: workspace.to_string(),
            path: format!("{}_{}.json", function.name, timestamp_str),
            name: function.name.clone(),
            args: function.type_args.clone(),
            inputs: Vec::<Vec<Felt>>::new(),
        }
    }

    /// Function to load the previous corpus if it exists
    pub fn load_from_file(filename: &String, workspace: &String) -> Self {
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
            workspace: workspace.to_string(),
            path: filename.clone(),
            name: data["name"].as_str().unwrap().to_string(),
            args: data["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|input_array| input_array.as_str().unwrap().to_string())
                .collect(),
            inputs: inputs,
        };
    }

    /// Function to dump the inputs corpus
    pub fn dump_json(&self) {
        let _ = create_dir(&self.workspace);
        let _ = create_dir(format!("{}/{}", &self.workspace, self.name.clone()));
        let _ = create_dir(format!("{}/{}/inputs", &self.workspace, self.name.clone()));
        //let _ = create_dir(self.workspace.input_folder);
        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");

        let mut inputs_ser = serde_json::Serializer::with_formatter(buf.clone(), formatter.clone());
        self.serialize(&mut inputs_ser).unwrap();
        let dump_file = format!(
            "{}/{}/inputs/{}",
            self.workspace,
            self.name.clone(),
            self.path
        );
        write(
            &dump_file,
            String::from_utf8(inputs_ser.into_inner()).unwrap(),
        )
        .expect("Failed to save input to disk");
    }
}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CrashFile {
    pub workspace: String,
    pub path: String,
    pub name: String,
    pub args: Vec<String>,
    pub crashes: Vec<Vec<Felt>>,
}

impl CrashFile {
    pub fn new_from_function(function: &Function, workspace: &String) -> Self {
        let d = SystemTime::now();
        // Create DateTime from SystemTime
        let datetime = DateTime::<Utc>::from(d);
        // Formats the combined date and time with the specified format string.
        let timestamp_str = datetime.format("%Y-%m-%d--%H:%M:%S").to_string();
        CrashFile {
            workspace: workspace.to_string(),
            path: format!("CRASHES_{}_{}.json", function.name, timestamp_str),
            name: function.name.clone(),
            args: function.type_args.clone(),
            crashes: Vec::<Vec<Felt>>::new(),
        }
    }

    /// Function to load a crashes corpus
    pub fn load_from_file(filename: &String, workspace: &String) -> Self {
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
            workspace: workspace.to_string(),
            path: filename.clone(),
            name: data["name"].as_str().unwrap().to_string(),
            args: data["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|input_array| input_array.as_str().unwrap().to_string())
                .collect(),
            crashes: crashes,
        };
    }

    /// Function to dump the crashes corpus
    pub fn dump_json(&self) {
        let _ = create_dir(&self.workspace);
        let _ = create_dir(format!("{}/{}", &self.workspace, self.name.clone()));
        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");

        let mut crashes_ser =
            serde_json::Serializer::with_formatter(buf.clone(), formatter.clone());
        self.serialize(&mut crashes_ser).unwrap();
        let dump_file = format!("{}/{}/{}", &self.workspace, self.name.clone(), self.path);
        write(
            &dump_file,
            String::from_utf8(crashes_ser.into_inner()).unwrap(),
        )
        .expect("Failed to save input to disk");
    }
}
