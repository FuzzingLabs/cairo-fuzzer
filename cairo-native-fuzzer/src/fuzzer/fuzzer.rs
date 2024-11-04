use std::path::PathBuf;
use std::sync::Arc;

use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::Program;
use cairo_native::utils::cairo_to_sierra;
use cairo_native::Value;
use starknet_types_core::felt::Felt;

use crate::mutator::argument_type::map_argument_type;
use crate::mutator::argument_type::ArgumentType;
use crate::mutator::mutator::Mutator;
use crate::runner::runner::CairoNativeRunner;
use crate::utils::get_function_by_id;

#[allow(dead_code)]
pub struct Fuzzer {
    program_path: PathBuf,
    entry_point: String,
    runner: CairoNativeRunner,
    sierra_program: Option<Arc<Program>>,
    params: Vec<Value>,
    entry_point_id: Option<FunctionId>,
    mutator: Mutator,
}

impl Fuzzer {
    pub fn new(program_path: PathBuf, entry_point: String) -> Self {
        Self {
            program_path,
            entry_point,
            runner: CairoNativeRunner::new(),
            sierra_program: None,
            params: Vec::new(),
            entry_point_id: None,
            mutator: Mutator::new(),
        }
    }

    /// Init the fuzzer
    /// - Compile Cairo code to Sierra
    /// - Find the entry id
    /// - Init the runner
    pub fn init(&mut self) -> Result<(), String> {
        self.convert_and_store_cairo_to_sierra()?;
        self.entry_point_id = Some(self.find_entry_point_id());
        self.runner
            .init(&self.entry_point_id, &self.sierra_program)?;
        self.generate_params();
        Ok(())
    }

    /// Compile the Cairo program to Sierra
    fn convert_and_store_cairo_to_sierra(&mut self) -> Result<(), String> {
        if self.sierra_program.is_none() {
            self.sierra_program = Some(cairo_to_sierra(&self.program_path));
        }
        Ok(())
    }

    /// Find the entry point id
    fn find_entry_point_id(&self) -> FunctionId {
        let sierra_program = self
            .sierra_program
            .as_ref()
            .expect("Sierra program not available");
        cairo_native::utils::find_function_id(sierra_program, &self.entry_point)
            .expect(&format!("Entry point '{}' not found", self.entry_point))
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
        let argument_types = self.get_function_arguments_types();
        self.params = argument_types
            .into_iter()
            .map(|arg_type| match arg_type {
                ArgumentType::Felt => Value::Felt252(Felt::from(0)),
                // TODO: Add support for other types
            })
            .collect();
    }

    /// Mutate a single function parameter
    pub fn mutate_param(&mut self, value: Value) -> Value {
        match value {
            Value::Felt252(felt) => {
                // Perform some mutation on the felt value
                // For now it's just a placeholder function
                let mutated_felt = felt;
                Value::Felt252(mutated_felt)
            }
            // TODO: Add support for other types
            _ => value,
        }
    }

    /// Mutate the parameters using the Mutator
    fn mutate_params(&mut self) {
        // Iterate through the current params and mutate each one
        for i in 0..self.params.len() {
            let mutated_value = self.mutate_param(self.params[i].clone());
            self.params[i] = mutated_value;
        }
    }

    /// Run the fuzzer
    /// We just use an infinite loop for now
    pub fn fuzz(&mut self) -> Result<(), String> {
        self.generate_params();
        self.mutate_params();
        loop {
            match self.runner.run_program(&self.params) {
                Ok(result) => {
                    println!("Cairo program was compiled and executed successfully.");
                    println!("{:?}", result);
                }
                Err(e) => eprintln!("Error during execution: {}", e),
            }
        }
    }
}
