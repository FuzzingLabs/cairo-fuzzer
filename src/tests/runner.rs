use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::types::program::Program;
use cairo_rs::types::relocatable::MaybeRelocatable;
use cairo_rs::types::relocatable::Relocatable;
use cairo_rs::vm::errors::vm_errors::VirtualMachineError;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;
use cairo_rs_py::cairo_runner::PyCairoRunner;
use num_bigint::BigInt;
use num_bigint::Sign;
use pyo3::marker::Python;
use pyo3::ToPyObject;
use serde_json::Value;
use std::env;
use std::fs;
use std::process;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    /// parse entrypoint number in the json
    pub entrypoint: String,
    pub num_args: u64,
    pub type_args: Vec<String>,
    pub hints: bool,
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
                        let new_function = Function {
                            entrypoint: pc,
                            hints: hints,
                            name: name,
                            num_args: size
                                .as_u64()
                                .expect("Failed to get number of arguments from json"),
                            type_args: get_type_args(members),
                        };
                        return Some(new_function);
                    }
                }
            }
        }
    }
    return None;
}

pub fn py_runner(
    json: &String,
    func_name: &String,
    entrypoint: &String,
    data: &Vec<u8>,
) -> Result<Option<Vec<(Relocatable, Relocatable)>>, VirtualMachineError> {
    let mut runner = PyCairoRunner::new(
        json.clone(),
        Some(func_name.clone()),
        None, //Some("plain".to_string()),
        false,
    )
    .unwrap();
    runner.initialize_segments();
    let mut ret = Vec::<(Relocatable, Relocatable)>::new();
    return Python::with_gil(
        |py| -> Result<Option<Vec<(Relocatable, Relocatable)>>, VirtualMachineError> {
            let args = data.to_object(py);
            // builtin init // add this to the beginning of the args
            let builtins = runner.get_program_builtins_initial_stack(py);
            match runner.run_from_entrypoint(
                py,
                py.eval(&entrypoint, None, None).unwrap(),
                args,
                None,
                None,
                Some(false),
                None,
                None,
            ) {
                Ok(_val) => {
                    let pyvm = runner.pyvm;
                    let vm = pyvm.get_vm();
                    match vm.clone().try_borrow() {
                        Ok(vm) => {
                            let trace = vm
                                .get_trace()
                                .expect("Failed to get running trace from the VM");
                            for i in trace {
                                ret.push((i.fp.clone(), i.pc.clone()));
                            }
                            return Ok(Some(ret));
                        }
                        Err(e) => {
                            println!("RUNNER ERROR -> {:?}", e);
                            return Ok(None);
                        }
                    }
                }
                Err(e) => {
                    println!("CAIRO-RS-PY RUNNER ERROR ===> {:?}", e);
                    return Ok(None);
                }
            }
            //runner.write_output();
        },
    );
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let contents = fs::read_to_string(&args[1]).expect("Should have been able to read the file");
    let function = match parse_json(&contents, &args[2]) {
        Some(func) => func,
        None => process::exit(1),
    };
    println!(" PC : {}", &function.entrypoint);
    let mut vec: Vec<u8> = Vec::new();
    for _ in 0..function.num_args {
        vec.push(5);
    }
    while true {
        py_runner(&contents, &function.name, &function.entrypoint, &vec);
    }
}
