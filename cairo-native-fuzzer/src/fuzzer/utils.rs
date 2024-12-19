use regex::Regex;
use std::sync::Arc;

use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::Program;
use colored::*;

use crate::mutator::argument_type::{map_argument_type, ArgumentType};
use crate::utils::get_cairo_native_version;
use crate::utils::get_function_by_id;

// Initialization message printed at fuzzer launch
const INIT_MESSAGE_FORMAT: &str = "
=============================================================================================================================================================
╔═╗ ┌─┐ ┬ ┬─┐ ┌───┐   ╔═╗ ┬ ┬ ┌─┐ ┌─┐ ┌─┐ ┬─┐      | Seed -- {}
║   ├─┤ │ ├┬┘ │2.0│───╠╣  │ │ ┌─┘ ┌─┘ ├┤  ├┬┘      | cairo-native version -- {}
╚═╝ ┴ ┴ ┴ ┴└─ └───┘   ╚   └─┘ └─┘ └─┘ └─┘ ┴└─      |
=============================================================================================================================================================
";

/// Print the initialization message
pub fn print_init_message(seed: u64) {
    let version = get_cairo_native_version();

    // Replace the first occurrence of {} with the seed value
    let re = Regex::new(r"\{\}").unwrap();
    let message = re.replace(INIT_MESSAGE_FORMAT, |_: &regex::Captures| seed.to_string());

    // Replace the next occurrence of {} with the version string
    let message = re.replace(&message, |_: &regex::Captures| version.to_string());

    println!("{}", message);
}

/// Returns a vector of the function parameter types
///
/// For example, given a function with the prototype:
/// ```
/// myfunction(a: felt252, b: felt252) -> felt252
/// ```
/// This function will return:
/// ```
/// [Felt, Felt]
/// ```
pub fn get_function_argument_types(
    sierra_program: &Option<Arc<Program>>,
    entry_point_id: &Option<FunctionId>,
) -> Vec<ArgumentType> {
    // Get the function from the Sierra program using the entry point id
    let func = match (sierra_program, entry_point_id) {
        (Some(program), Some(entry_point_id)) => get_function_by_id(program, entry_point_id),
        _ => None,
    };

    // Iterate through entry point arguments and map their types to a type supported by the fuzzer
    if let Some(func) = func {
        let argument_types: Vec<ArgumentType> = func
            .signature
            .param_types
            .iter()
            .filter_map(|param_type| {
                if let Some(debug_name) = &param_type.debug_name {
                    // Map param_type to an `ArgumentType`
                    // For now we only handle felt252
                    return map_argument_type(debug_name);
                }
                None
            })
            .collect();
        argument_types
    } else {
        Vec::new()
    }
}

/// Print the contract functions prototypes
pub fn print_contract_functions(sierra_program: &Option<Arc<Program>>) {
    println!("Contract functions :\n");

    if let Some(program) = sierra_program {
        for function in &program.funcs {
            let function_name = function
                .id
                .debug_name
                .as_ref()
                .expect("Function name not found")
                .to_string();

            let signature = &function.signature;

            // Collect parameter types
            let param_types: Vec<String> = signature
                .param_types
                .iter()
                .map(|param| {
                    param
                        .debug_name
                        .as_ref()
                        .expect("Parameter name not found")
                        .to_string()
                })
                .collect();

            // Collect return types
            let ret_types: Vec<String> = signature
                .ret_types
                .iter()
                .map(|ret_type| {
                    ret_type
                        .debug_name
                        .as_ref()
                        .expect("Return type name not found")
                        .to_string()
                })
                .collect();

            // Format the prototype
            let prototype = format!(
                "{} ({}) -> ({})",
                function_name.bold().white(),
                param_types.join(", ").green(),
                ret_types.join(", ").cyan()
            );

            // Print the contract functions
            println!("- {}", prototype);
        }
    }
}

/// Find the entry point id
pub fn find_entry_point_id(sierra_program: &Option<Arc<Program>>, entry_point: &str) -> FunctionId {
    let sierra_program = sierra_program
        .as_ref()
        .expect("Sierra program not available");
    cairo_native::utils::find_function_id(sierra_program, entry_point)
        .expect(&format!("Entry point '{}' not found", entry_point))
        .clone()
}
