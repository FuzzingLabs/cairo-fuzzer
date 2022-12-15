use crate::cairo_vm::cairo_types::Felt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputCorpus {
    pub name: String,
    pub args: Vec<String>,
    pub inputs: Vec<Vec<Felt>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrashCorpus {
    pub name: String,
    pub args: Vec<String>,
    pub crashes: Vec<Vec<Felt>>,
}
/*
/// Function to load the previous corpus if it exists
 pub fn load_inputs_corpus(fuzzing_data: &Arc<FuzzingData>, mut filename: &String) -> InputCorpus {
    let mut inputs_corpus = InputCorpus {
        name: fuzzing_data.function.name.clone(),
        args: fuzzing_data.function.type_args.clone(),
        inputs: Vec::<Vec<Felt>>::new(),
    };
    filename = if filename.len() == 0 {
        &format!("inputs_corpus/{}_inputs.json", inputs_corpus.name)
    } else {
        filename
    };
    if Path::new(&filename).is_file() {
        let contents = fs::read_to_string(filename)
            .expect("Should have been able to read the file");
        let data: Value = serde_json::from_str(&contents).expect("JSON was not well-formatted");
        // Load old inputs to prevent overwriting and to use it as a dictionary for the mutator
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
        inputs_corpus.inputs.extend(inputs);
    }
    return inputs_corpus;
} */

// 1st case - replay multiple inputs (.json) or multiple jsons with inputs
// 2nd case - replay multiple crashes (.json) or multiple json with crashes
// 3rd case - replay single input or crash (vector with data)
/*
/// Function to load the crashes inputs
 pub fn load_crashes_corpus(fuzzing_data: &Arc<FuzzingData>, mut filename: &String) -> CrashCorpus {
    let mut crashes_corpus = CrashCorpus {
        name: fuzzing_data.function.name.clone(),
        args: fuzzing_data.function.type_args.clone(),
        crashes: Vec::<Vec<Felt>>::new(),
    };
    filename = if filename.len() == 0 {
        &format!("crashes_corpus/{}_crashes.json", crashes_corpus.name)
    } else {
        filename
    };
    if Path::new(&filename).is_file() {
        let contents = fs::read_to_string(filename)
            .expect("Should have been able to read the file");
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
        crashes_corpus.crashes.extend(crashes);
    }
    return crashes_corpus;
} */
