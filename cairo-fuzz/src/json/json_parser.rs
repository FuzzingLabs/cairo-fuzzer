use serde_json::Value;

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

pub fn parse_json(data: &String, function_name: &String) -> Option<Function> {
    let data: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
    let identifiers = &data["identifiers"];
    for (key, value) in identifiers.as_object().unwrap() {
        let name = key.split(".").last().unwrap().to_string();
        if value["type"] == "function" && &name == function_name {
            if let Some(_field) = identifiers.get(format!("{}.Args", key)) {
                let new_function = Function {
                    name: name,
                    num_args: identifiers[format!("{}.Args", key)]["size"]
                        .as_u64()
                        .unwrap(),
                    type_args: get_type_args(&identifiers[format!("{}.Args", key)]["members"]),
                };
                return Some(new_function);
            }
        }
    }
    return None;
}
