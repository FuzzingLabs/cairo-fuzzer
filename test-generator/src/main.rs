use std::collections::HashSet;
use std::env;
use std::fs;
use std::process;
use std::str::FromStr;

use serde::Serialize;

use sierra_analyzer_lib::sierra_program::SierraProgram;
use sierra_analyzer_lib::sym_exec::sym_exec::generate_test_cases_for_function;

/// Struct representing the fuzzing data
#[derive(Serialize)]
struct FuzzingData {
    workspace: String,
    path: String,
    name: String,
    args: Vec<String>,
    inputs: Vec<Vec<Value>>,
}

/// Struct representing a value in the fuzzing data
#[derive(Serialize)]
struct Value {
    value: ValueData,
}

/// Struct representing the value data
#[derive(Serialize)]
struct ValueData {
    val: Vec<i64>,
}

/// Prints the names of all available functions in the decompiler
fn print_function_names(decompiler: &sierra_analyzer_lib::decompiler::decompiler::Decompiler) {
    println!("Available functions:");
    for function in &decompiler.functions {
        if let Some(prototype) = &function.prototype {
            let function_name = extract_function_name(prototype);
            println!("\t- {}", function_name);
        }
    }
}

/// Extracts the function name from the prototype string
fn extract_function_name(prototype: &str) -> String {
    let stripped_prototype = &prototype[5..];
    if let Some(first_space_index) = stripped_prototype.find('(') {
        return stripped_prototype[..first_space_index].trim().to_string();
    }
    String::new()
}

/// Parses the result of generate_test_cases_for_function and returns a vector of vectors of integer inputs
fn get_integers_inputs(test_cases: &str) -> Vec<Vec<i64>> {
    let unique_results: HashSet<String> = test_cases.lines().map(|line| line.to_string()).collect();
    unique_results
        .iter()
        .map(|line| parse_line_inputs(line))
        .collect()
}

/// Parses a single line of test cases and returns a vector of integer inputs
fn parse_line_inputs(line: &str) -> Vec<i64> {
    let parts: Vec<&str> = line.split(", ").collect();
    parts
        .iter()
        .filter_map(|part| {
            let key_value: Vec<&str> = part.split(": ").collect();
            if key_value.len() == 2 {
                if let Ok(value) = i64::from_str(key_value[1]) {
                    return Some(value);
                }
            }
            None
        })
        .collect()
}

/// Generates the fuzzing data for a given function
fn generate_fuzzing_data(
    function: &mut sierra_analyzer_lib::decompiler::function::Function,
    declared_libfuncs_names: Vec<String>,
    workspace: &str,
    path: &str,
    name: &str,
) -> FuzzingData {
    let test_cases = generate_test_cases_for_function(function, declared_libfuncs_names);
    let integer_inputs = get_integers_inputs(&test_cases);
    let arg_count = function.arguments.len();
    let args = vec!["felt".to_string(); arg_count];
    let inputs = convert_integer_inputs_to_values(integer_inputs);

    FuzzingData {
        workspace: workspace.to_string(),
        path: path.to_string(),
        name: name.to_string(),
        args,
        inputs,
    }
}

/// Converts integer inputs to the desired JSON format
fn convert_integer_inputs_to_values(integer_inputs: Vec<Vec<i64>>) -> Vec<Vec<Value>> {
    integer_inputs
        .iter()
        .map(|inputs| {
            inputs
                .iter()
                .map(|&input| Value {
                    value: ValueData { val: vec![input] },
                })
                .collect()
        })
        .collect()
}

/// Main function to handle command-line arguments and generate fuzzing data
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: Please provide a file path as an argument.");
        process::exit(1);
    }

    let file_path = &args[1];

    // Read the content of the Sierra program file
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path, e);
            process::exit(1);
        }
    };

    // Initialize a new SierraProgram with the content of the .sierra file
    let program = SierraProgram::new(content);

    // Disable verbose output for the decompiler
    let verbose_output = false;

    // Create a decompiler instance for the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the Sierra program
    let use_color = false;
    decompiler.decompile(use_color);

    if args.len() == 2 {
        // No specific function specified, print all available functions
        print_function_names(&decompiler);
    } else {
        // Specific function specified, generate test cases for that function
        let function_name = &args[2];
        let mut found = false;

        for function in &mut decompiler.functions {
            if let Some(prototype) = &function.prototype {
                let name = extract_function_name(prototype);
                if name == *function_name {
                    let fuzzing_data = generate_fuzzing_data(
                        function,
                        decompiler.declared_libfuncs_names.clone(),
                        "fuzzer_workspace",
                        "input_file",
                        "Fuzz_one",
                    );

                    // Serialize the data to JSON and print it
                    let json_output = serde_json::to_string_pretty(&fuzzing_data).unwrap();
                    println!("{}", json_output);

                    found = true;
                    break;
                }
            }
        }

        if !found {
            eprintln!("Error: Function '{}' not found.", function_name);
            process::exit(1);
        }
    }
}
