use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub selector_idx: usize,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}
#[derive(Debug)]
pub struct AbiFunction {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

fn extract_function(tmp: &serde_json::Map<String, Value>) -> AbiFunction {
    let name = tmp
        .get("name")
        .expect("Could not get name of function from the abi")
        .as_str()
        .expect("Could not convert to str")
        .to_string();
    let inputs_data = tmp
        .get("inputs")
        .expect("Could not get inputs from the abi")
        .as_array()
        .expect("Could not convert inputs to array");
    let mut inputs: Vec<String> = vec![];
    for input in inputs_data {
        inputs.push(
            input
                .get("type")
                .expect("Could not get type from input")
                .as_str()
                .expect("Could not convert to str")
                .to_string(),
        );
    }
    let outputs_data = tmp
        .get("outputs")
        .expect("Could not get outputs from the abi")
        .as_array()
        .expect("Could not convert outputs to array");
    let mut outputs: Vec<String> = vec![];
    for output in outputs_data {
        outputs.push(
            output
                .get("type")
                .expect("Could not get type from input")
                .as_str()
                .expect("Could not convert to str")
                .to_string(),
        );
    }
    return AbiFunction {
        name: name,
        inputs: inputs,
        outputs: outputs,
    };
}

fn search_for_function(data: &Vec<Value>) -> Vec<AbiFunction> {
    let mut res: Vec<AbiFunction> = vec![];
    for obj in data {
        let tmp: &serde_json::Map<String, Value> = obj
            .as_object()
            .expect("could not convert abi obj to object");
        let obj_type = tmp
            .get("type")
            .expect("Could not get abi object type")
            .as_str()
            .expect("Could not convert to str");
        if obj_type == "function" {
            let state_mutability = tmp
                .get("state_mutability")
                .expect("Could not get state_mutability")
                .as_str()
                .expect("Could not convert to str");
            if state_mutability == "external" {
                res.push(extract_function(tmp));
            }
        }
        if obj_type == "interface" {
            let items = tmp
                .get("items")
                .expect("Could not get interface items")
                .as_array()
                .expect("Could not convert to str");
            res.append(&mut search_for_function(items));
        }
    }
    return res;
}

fn get_abi(data: &Value) -> Vec<AbiFunction> {
    let mut res: Vec<AbiFunction> = vec![];
    if let Some(abi) = data.get("abi") {
        let abi = abi.as_array().expect("Could not convert abi to array");
        res.append(&mut search_for_function(abi))
    }
    res
}

pub fn get_function_from_json(data: &String, function_name: &String) -> Option<Function> {
    let data: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
    let abi = get_abi(&data);
    if let Some(_types) = data.get("entry_points_by_type") {
        let mut idx: usize = 0;
        for function_abi in abi {
            if function_name == &*function_abi.name {
                return Some(Function {
                    name: function_abi.name,
                    selector_idx: idx,
                    inputs: function_abi.inputs,
                    outputs: function_abi.outputs,
                });
            }
            idx += 1;
        }
    };
    return None;
}

pub fn analyze_json(data: &String) {
    println!("Running json analyzer ... \n");
    let data: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
    let abi = get_abi(&data);
    if let Some(_types) = data.get("entry_points_by_type") {
        let mut idx: usize = 0;
        for function_abi in abi {
            let func = Function {
                name: function_abi.name,
                selector_idx: idx,
                inputs: function_abi.inputs,
                outputs: function_abi.outputs,
            };
            let mut prototype = "".to_string();
            prototype += &func.name;
            prototype += "(";
            let mut input_idx = 0;
            for input in &func.inputs {
                prototype += &input;
                if input_idx != func.inputs.len() - 1 {
                    prototype += ", ";
                }
                input_idx += 1;
            }
            prototype += ")";
            if &func.outputs.len() > &0 {
                prototype += " -> ";
            }
            let mut output_idx = 0;
            for output in &func.outputs {
                prototype += &output;
                if output_idx != func.outputs.len() - 1 {
                    prototype += " ,";
                }
                output_idx += 1;
            }
            println!("{}\n", prototype);
            idx += 1;
        }
    }
}

// To test before deploying on master
pub fn get_proptesting_functions(data: &String) -> Vec<String> {
    let content: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
    let mut functions: Vec<String> = vec![];
    let abi = get_abi(&content);
    for func in abi {
        if func.name.starts_with("Fuzz_") {
            functions.push(func.name);
        }
    }
    functions
}
