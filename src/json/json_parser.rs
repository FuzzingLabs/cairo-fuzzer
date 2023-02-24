use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub entrypoint: String,
    pub num_args: u64,
    pub type_args: Vec<String>,
    pub hints: bool,
    pub decorators: Vec<String>,
    pub _starknet: bool,
}

/// Function that returns a vector of the args type of the function the user want to fuzz
fn get_type_args(members: &Value) -> Vec<String> {
    let mut type_args = Vec::<String>::new();
    for (_, value) in members
        .as_object()
        .expect("Failed get member type_args as object from json")
    {
        type_args.push(value["cairo_type"].to_string().replace("\"", ""));
    }
    return type_args;
}

/// Function that return a vector of the decoratos of a function
fn get_decorators(decorators: &Value) -> Vec<String> {
    let mut decorators_list = Vec::<String>::new();
    if let Some(data) = decorators.as_array() {
        for elem in data {
            decorators_list.push(elem.to_string().replace("\"", ""));
        }
    }
    return decorators_list;
}

fn get_pc_from_wrapper(identifiers: &Value, function_name: &String) -> String {
    for (key, value) in identifiers
    .as_object()
    .expect("Failed to get identifier from json")
    {
        let key_split = key.split(".").collect::<Vec<&str>>();
        if value["type"] == "function" && key.contains("wrapper") && key_split.len() == 2 {
            let name = key_split[key_split.len() - 1];
            if name == function_name {
                return value["pc"].to_string();
            }
        
    }
}
    return "".to_string();
}

/// Function to parse starknet json artifact
pub fn parse_starknet_json(data: &String, function_name: &String) -> Option<Function> {
    let mut starknet = false;
    let mut data: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
    if let Some(program) = data.get("program") {
        data = program.clone();
        starknet = true;
    }
    let hints = if let Some(field) = data.get("hints") {
        field.as_object().unwrap().len() != 0
    } else {
        false
    };
    if let Some(identifiers) = data.get("identifiers") {
        for (key, value) in identifiers
            .as_object()
            .expect("Failed to get identifier from json")
        {
            let key_split = key.split(".").collect::<Vec<&str>>();
            if value["type"] == "function" && key.contains("main") && key_split.len() == 2 {
                let name = key_split[key_split.len() - 1];
                let pc = get_pc_from_wrapper(identifiers, function_name);
                if pc.is_empty() {
                    eprintln!("Error : Could not get PC");
                    return None;
                }
                let mut decorators = Vec::<String>::new();
                if let Some(decorators_data) = value.get("decorators") {
                    decorators.append(&mut get_decorators(decorators_data));
                }
                if let Some(identifiers_key) = identifiers.get(format!("{}.Args", key)) {
                    if let (Some(size), Some(members)) =
                        (identifiers_key.get("size"), identifiers_key.get("members"))
                    {
                        if &name.to_string() == function_name && decorators[0] == "external" {
                            return Some(Function {
                                _starknet: starknet,
                                entrypoint: pc,
                                hints: hints,
                                name: name.to_string(),
                                num_args: size
                                    .as_u64()
                                    .expect("Failed to get number of arguments from json"),
                                decorators: decorators,
                                type_args: get_type_args(members),
                            });
                        }
                    }
                }
            }
        }
    }
    eprintln!("Error : Could not get function");
    return None;
}

/// Function to parse cairo json artifact
pub fn parse_json(data: &String, function_name: &String) -> Option<Function> {
    let starknet = false;
    let data: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
    let hints = if let Some(field) = data.get("hints") {
        field.as_object().unwrap().len() != 0
    } else {
        false
    };
    if let Some(identifiers) = data.get("identifiers") {
        for (key, value) in identifiers
            .as_object()
            .expect("Failed to get identifier from json")
        {
            let name = key.split(".").last().unwrap().to_string();
            if value["type"] == "function" && &name == function_name {
                let pc = value["pc"].to_string();
                if let Some(identifiers_key) = identifiers.get(format!("{}.Args", key)) {
                    if let (Some(size), Some(members)) =
                        (identifiers_key.get("size"), identifiers_key.get("members"))
                    {
                        return Some(Function {
                            decorators: Vec::new(),
                            _starknet: starknet,
                            entrypoint: pc,
                            hints: hints,
                            name: name,
                            num_args: size
                                .as_u64()
                                .expect("Failed to get number of arguments from json"),
                            type_args: get_type_args(members),
                        });
                    }
                }
            }
        }
    }
    return None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;
    use std::fs;
    #[test]
    fn test_good_json() {
        let filename = "tests/fuzzinglabs.json";
        let contents = fs::read_to_string(&filename.to_string())
            .expect("Should have been able to read the file");
        let function = parse_json(&contents, &"test_symbolic_execution".to_string());
        if let Some(ok_function) = function {
            assert_eq!(ok_function.name, "test_symbolic_execution".to_string());
            assert_eq!(ok_function.num_args, 11);
            assert_eq!(
                ok_function.type_args,
                vec![
                    "felt", "felt", "felt", "felt", "felt", "felt", "felt", "felt", "felt", "felt",
                    "felt"
                ]
            );
        } else {
            panic!("Should be parsed properly")
        }
    }
    #[test]
    fn test_bad_json() {
        let content = r###"{
            "name": "test_symbolic_execution"}"###;
        let function = parse_json(&content.to_string(), &"test_symbolic_execution".to_string());
        if let Some(_function) = function {
            panic!("should not be parsed");
        }
    }
    #[test]
    fn test_good_json_bad_function_name() {
        let filename = "tests/fuzzinglabs.json";
        let contents = fs::read_to_string(&filename.to_string())
            .expect("Should have been able to read the file");
        let function = parse_json(&contents, &"bad_function_name".to_string());
        if let Some(_function) = function {
            panic!("should not be parser properly")
        }
    }
}
