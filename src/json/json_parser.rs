use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub num_args: u64,
    pub type_args: Vec<String>,
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

pub fn parse_json(data: &String, function_name: &String) -> Option<Function> {
    let data: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
    if let Some(identifiers) = data.get("identifiers") {
        for (key, value) in identifiers.as_object().expect("Failed to get identifier from json") {
            let name = key.split(".").last().unwrap().to_string();
            if value["type"] == "function" && &name == function_name {
                if let Some(identifiers_key) = identifiers.get(format!("{}.Args", key)) {
                    if let (Some(size), Some(members)) = (identifiers_key.get("size"), identifiers_key.get("members")) {
                        let new_function = Function {
                            name: name,
                            num_args: size
                                .as_u64()
                                .expect("Failed to get number of arguments from json"),
                            type_args: get_type_args(
                                members,
                            ),
                        };
                        return Some(new_function);  
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
