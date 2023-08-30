use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub selector_idx: usize,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub starknet: bool,
}
#[derive(Debug)]
pub struct AbiFunction {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

fn get_abi(data: &Value) -> Vec<AbiFunction> {
    let mut res: Vec<AbiFunction> = vec![];
    if let Some(abi) = data.get("abi") {
        //println!("{:?}", abi);
        for obj in abi.as_array().expect("Could not convert abi to array") {
            let tmp = obj
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
                    res.push(AbiFunction {
                        name: name,
                        inputs: inputs,
                        outputs: outputs,
                    });
                }
            }
        }
    }
    res
}

pub fn parse_json(data: &String, function_name: &String) -> Option<Function> {
    let data: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
    let abi = get_abi(&data);
    if let Some(_types) = data.get("entry_points_by_type") {
        /*         let extetests/fuzzinglabs.jsonrnal_functions = types
        .get("EXTERNAL")
        .expect("Could not get external functions")
        .as_array()
        .expect("Could not convert external functions to array"); */
        let mut idx: usize = 0;
        for function_abi in abi {
            if function_name == &*function_abi.name {
                return Some(Function {
                    name: function_abi.name,
                    selector_idx: idx,
                    inputs: function_abi.inputs,
                    outputs: function_abi.outputs,
                    starknet: true,
                });
            }
            idx += 1;
        }
    };
    return None;
}

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
