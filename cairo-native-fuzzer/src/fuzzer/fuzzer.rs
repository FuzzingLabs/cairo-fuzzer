use std::path::PathBuf;
use std::sync::Arc;

use cairo_lang_compiler::CompilerConfig;
use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::Program;
use cairo_lang_starknet::compile::compile_path;
use colored::*;
use starknet_types_core::felt::Felt;

use crate::mutator::argument_type::map_argument_type;
use crate::mutator::argument_type::ArgumentType;
use crate::mutator::basic_mutator::Mutator;
use crate::runner::runner::CairoNativeRunner;
use crate::utils::get_function_by_id;

#[allow(dead_code)]
pub struct Fuzzer {
    program_path: PathBuf,
    entry_point: Option<String>,
    runner: CairoNativeRunner,
    sierra_program: Option<Arc<Program>>,
    params: Vec<Felt>,
    entry_point_id: Option<FunctionId>,
    mutator: Mutator,
    argument_types: Vec<ArgumentType>,
}

impl Fuzzer {
    pub fn new(program_path: PathBuf, entry_point: Option<String>) -> Self {
        Self {
            program_path,
            entry_point,
            runner: CairoNativeRunner::new(),
            sierra_program: None,
            params: Vec::new(),
            entry_point_id: None,
            mutator: Mutator::new(),
            argument_types: Vec::new(),
        }
    }

    /// Init the fuzzer
    /// - Compile Cairo code to Sierra
    /// - Find the entry id
    /// - Init the runner
    pub fn init(&mut self) -> Result<(), String> {
        self.convert_and_store_cairo_to_sierra()?;
        if let Some(ref entry_point) = self.entry_point {
            self.entry_point_id = Some(self.find_entry_point_id(entry_point));
        }
        self.runner
            .init(&self.entry_point_id, &self.sierra_program)?;
        Ok(())
    }

    /// Print the contract functions prototypes
    pub fn print_functions_prototypes(&self) {
        println!("Contract functions :\n");

        for function in &self.sierra_program.clone().unwrap().funcs {
            let function_name = function
                .id
                .debug_name
                .as_ref()
                .map_or_else(|| "unknown".to_string(), |name| name.to_string());

            let signature = &function.signature;

            // Collect parameter types
            let param_types: Vec<String> = signature
                .param_types
                .iter()
                .map(|param| {
                    param
                        .debug_name
                        .as_ref()
                        .map_or_else(|| "unknown".to_string(), |name| name.to_string())
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
                        .map_or_else(|| "unknown".to_string(), |name| name.to_string())
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

    /// Compile the Cairo program to Sierra
    fn convert_and_store_cairo_to_sierra(&mut self) -> Result<(), String> {
        let contract = compile_path(
            &self.program_path,
            None,
            CompilerConfig {
                replace_ids: true,
                ..Default::default()
            },
        )
        .map_err(|e| format!("Failed to compile Cairo program: {}", e))?;

        let sierra_program = contract
            .extract_sierra_program()
            .map_err(|e| format!("Failed to extract Sierra program: {}", e))?;
        self.sierra_program = Some(Arc::new(sierra_program));
        Ok(())
    }

    /// Find the entry point id
    fn find_entry_point_id(&self, entry_point: &str) -> FunctionId {
        let sierra_program = self
            .sierra_program
            .as_ref()
            .expect("Sierra program not available");
        cairo_native::utils::find_function_id(sierra_program, entry_point)
            .expect(&format!("Entry point '{}' not found", entry_point))
            .clone()
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
    pub fn get_function_arguments_types(&self) -> Vec<ArgumentType> {
        let func = match (&self.sierra_program, &self.entry_point_id) {
            (Some(program), Some(entry_point_id)) => get_function_by_id(program, entry_point_id),
            _ => None,
        };

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

    /// Generate params based on the function argument types
    pub fn generate_params(&mut self) {
        self.params = self
            .argument_types
            .iter()
            .map(|arg_type| match arg_type {
                ArgumentType::Felt => Felt::from(0),
                // TODO: Add support for other types
            })
            .collect();
    }

    /// Mutate a single function parameter
    pub fn mutate_param(&mut self, value: Felt) -> Felt {
        // Use the Mutator to mutate the felt value
        self.mutator.mutate(value)
    }

    /// Mutate the parameters using the Mutator
    fn mutate_params(&mut self) {
        // Iterate through the current params and mutate each one
        for i in 0..self.params.len() {
            let mutated_value = self.mutate_param(self.params[i]);
            self.params[i] = mutated_value;
        }
    }

    /// Run the fuzzer
    /// We just use an infinite loop for now
    pub fn fuzz(&mut self) -> Result<(), String> {
        self.argument_types = self.get_function_arguments_types();
        self.generate_params();

        loop {
            match self.runner.run_program(&self.params) {
                Ok(result) => {
                    println!("Results : {:?}", result);
                }
                Err(e) => eprintln!("Error during execution: {}", e),
            }
            self.mutate_params();
        }
    }
}
