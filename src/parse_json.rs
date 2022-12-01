use serde_json::{Result, Value};
use std::fs;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub num_args: u64,
    pub type_args: Vec<String>,
}

pub fn get_type_args(members: &Value) -> Vec<String> {
    let mut type_args = Vec::<String>::new();
    for (_key, value) in members.as_object().unwrap() {
        type_args.push(value["cairo_type"].to_string());
    }
    return type_args;
}

pub fn parse_json(filename: &String) -> Vec<Function> {
    println!("====> Parsing file {}", filename);
    println!("");
    let data = fs::read_to_string(filename).expect("Unable to read file");
    let data: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
    let identifiers = &data["identifiers"];
    let mut functions: Vec<Function> = Vec::<Function>::new();
    for (key, value) in identifiers.as_object().unwrap() {
        if value["type"] == "function" {
            if let Some(_field) = identifiers.get(format!("{}.Args", key)) {
                let new_function = Function {
                    name: key.split(".").last().unwrap().to_string(),
                    num_args: identifiers[format!("{}.Args", key)]["size"]
                        .as_u64()
                        .unwrap(),
                    type_args: get_type_args(&identifiers[format!("{}.Args", key)]["members"]),
                };
                functions.push(new_function);
            }
        }
    }
    //println!("{:?}", functions);
    return functions;
}
