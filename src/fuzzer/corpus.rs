use crate::{cairo_vm::cairo_types::Felt, json::json_parser::Function};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, fs::create_dir, fs::write, path::Path, process, time::SystemTime};

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
            .expect("Failed to get inputs from inputfile")
            .iter()
            .map(|input_array| {
                input_array
                    .as_array()
                    .expect("Failed to get input as array")
                    .iter()
                    .map(|input| input.as_u64().expect("Failed to get input as u64") as Felt)
                    .collect()
            })
            .collect();

        return InputFile {
            workspace: workspace.to_string(),
            path: filename.clone(),
            name: data["name"]
                .as_str()
                .expect("Failed to get name from inputfile")
                .to_string(),
            args: data["args"]
                .as_array()
                .expect("Failed to get args from input file as array")
                .iter()
                .map(|input_array| {
                    input_array
                        .as_str()
                        .expect("Failed to get input array as string")
                        .to_string()
                })
                .collect(),
            inputs: inputs,
        };
    }

    pub fn load_from_folder(foldername: &String, workspace: &String) -> Self {
        let folder = Path::new(&foldername);
        let function_name = foldername
            .clone()
            .split('/')
            .last()
            .expect("Failed to split foldername")
            .to_string();
        let mut args: Option<Vec<String>> = None;
        let mut inputs: Vec<Vec<Felt>> = Vec::new();
        // Check if the path is a directory
        if folder.is_dir() {
            // Iterate over the entries in the directory
            for entry in fs::read_dir(folder).expect("Failed to read directory") {
                let entry = entry.expect("Failed to get entry");
                let path = entry.path();
                // Check if the entry is a file
                if path.is_file() {
                    // Read the file and do something with its contents
                    let contents =
                        fs::read_to_string(&path).expect("Failed to read string from the file");
                    let data: Value =
                        serde_json::from_str(&contents).expect("JSON was not well-formatted");
                    let args_data: Vec<String> = data["args"]
                        .as_array()
                        .expect("Failed to get args from input file as array")
                        .iter()
                        .map(|input_array| {
                            input_array
                                .as_str()
                                .expect("Failed to get input array as string")
                                .to_string()
                        })
                        .collect();
                    if args.is_none() {
                        args = Some(args_data);
                    } else {
                        if let Some(args_to_compare) = args.clone() {
                            if args_to_compare != args_data {
                                println!("Uncompatible inputs files");
                                process::exit(1);
                            }
                        }
                    }
                    let mut data_inputs: Vec<Vec<Felt>> = data["inputs"]
                        .as_array()
                        .expect("Failed to get inputs from inputfile")
                        .iter()
                        .map(|input_array| {
                            input_array
                                .as_array()
                                .expect("Failed to get input as array")
                                .iter()
                                .map(|input| {
                                    input.as_u64().expect("Failed to get input as u64") as Felt
                                })
                                .collect()
                        })
                        .collect();
                    inputs.append(&mut data_inputs);
                }
            }
        }
        let d = SystemTime::now();
        // Create DateTime from SystemTime
        let datetime = DateTime::<Utc>::from(d);
        // Formats the combined date and time with the specified format string.
        let timestamp_str = datetime.format("%Y-%m-%d--%H:%M:%S").to_string();
        let data_args = if let Some(content) = args {
            content
        } else {
            Vec::new()
        };
        return InputFile {
            workspace: workspace.to_string(),
            path: format!("{}_{}.json", function_name.clone(), timestamp_str),
            name: function_name.clone(),
            args: data_args,
            inputs: inputs,
        };
    }
    /// Function to dump the inputs corpus
    pub fn dump_json(&self) {
        let _ = create_dir(&self.workspace);
        let _ = create_dir(format!("{}/{}", &self.workspace, &self.name));
        let _ = create_dir(format!("{}/{}/inputs", &self.workspace, &self.name));
        //let _ = create_dir(self.workspace.input_folder);
        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");

        let mut inputs_ser = serde_json::Serializer::with_formatter(buf.clone(), formatter.clone());
        self.serialize(&mut inputs_ser)
            .expect("Failed to serialize");
        let dump_file = format!(
            "{}/{}/inputs/{}",
            self.workspace,
            self.name.clone(),
            self.path
        );
        write(
            &dump_file,
            String::from_utf8(inputs_ser.into_inner()).expect("Failed to dump string as utf8"),
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
            .expect("Failed to get inputs from crashfile")
            .iter()
            .map(|input_array| {
                input_array
                    .as_array()
                    .expect("Failed to get input as array")
                    .iter()
                    .map(|input| input.as_u64().expect("Failed to get input as u64") as Felt)
                    .collect()
            })
            .collect();

        return CrashFile {
            workspace: workspace.to_string(),
            path: filename.clone(),
            name: data["name"]
                .as_str()
                .expect("Failed to get name from crashfile")
                .to_string(),
            args: data["args"]
                .as_array()
                .expect("Failed to get args from input file as array")
                .iter()
                .map(|input_array| {
                    input_array
                        .as_str()
                        .expect("Failed to get input array as string")
                        .to_string()
                })
                .collect(),
            crashes: crashes,
        };
    }

    pub fn load_from_folder(foldername: &String, workspace: &String) -> Self {
        let folder = Path::new(&foldername);
        let function_name = foldername
            .clone()
            .split('/')
            .last()
            .expect("Failed to split foldername")
            .to_string();
        let mut args: Option<Vec<String>> = None;
        let mut inputs: Vec<Vec<Felt>> = Vec::new();
        // Check if the path is a directory
        if folder.is_dir() {
            // Iterate over the entries in the directory
            for entry in fs::read_dir(folder).expect("Failed to read directory") {
                let entry = entry.expect("Failed to get entry");
                let path = entry.path();
                // Check if the entry is a file
                if path.is_file() {
                    // Read the file and do something with its contents
                    let contents =
                        fs::read_to_string(&path).expect("Failed to read string from the file");
                    let data: Value =
                        serde_json::from_str(&contents).expect("JSON was not well-formatted");
                    let args_data: Vec<String> = data["args"]
                        .as_array()
                        .expect("Failed to get args from input file as array")
                        .iter()
                        .map(|input_array| {
                            input_array
                                .as_str()
                                .expect("Failed to get input array as string")
                                .to_string()
                        })
                        .collect();
                    if args.is_none() {
                        args = Some(args_data);
                    } else {
                        if let Some(args_to_compare) = args.clone() {
                            if args_to_compare != args_data {
                                println!("Uncompatible inputs files");
                                process::exit(1);
                            }
                        }
                    }
                    let mut data_inputs: Vec<Vec<Felt>> = data["inputs"]
                        .as_array()
                        .expect("Failed to get inputs from inputfile")
                        .iter()
                        .map(|input_array| {
                            input_array
                                .as_array()
                                .expect("Failed to get input as array")
                                .iter()
                                .map(|input| {
                                    input.as_u64().expect("Failed to get input as u64") as Felt
                                })
                                .collect()
                        })
                        .collect();
                    inputs.append(&mut data_inputs);
                }
            }
        }
        let d = SystemTime::now();
        // Create DateTime from SystemTime
        let datetime = DateTime::<Utc>::from(d);
        // Formats the combined date and time with the specified format string.
        let timestamp_str = datetime.format("%Y-%m-%d--%H:%M:%S").to_string();
        let data_args = if let Some(content) = args {
            content
        } else {
            Vec::new()
        };
        return CrashFile {
            workspace: workspace.to_string(),
            path: format!("{}_{}.json", function_name.clone(), timestamp_str),
            name: function_name.clone(),
            args: data_args,
            crashes: inputs,
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
        self.serialize(&mut crashes_ser)
            .expect("Failed to serialize");
        let dump_file = format!("{}/{}/{}", &self.workspace, self.name.clone(), self.path);
        write(
            &dump_file,
            String::from_utf8(crashes_ser.into_inner()).expect("Failed to dump string as utf8"),
        )
        .expect("Failed to save input to disk");
    }
}
