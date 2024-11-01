use std::path::PathBuf;
use std::sync::Arc;

use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::Program;
use cairo_native::utils::cairo_to_sierra;
use cairo_native::Value;

use crate::mutator::argument_type::map_argument_type;
use crate::mutator::argument_type::ArgumentType;
use crate::runner::runner::CairoNativeRunner;
use crate::utils::get_function_by_id;

pub struct Fuzzer {
    program_path: PathBuf,
    entry_point: String,
    runner: CairoNativeRunner,
    sierra_program: Option<Arc<Program>>,
    params: Vec<Value>,
    entry_point_id: Option<FunctionId>,
}

impl Fuzzer {
    pub fn new(program_path: PathBuf, entry_point: String, params: Vec<Value>) -> Self {
        Self {
            program_path,
            entry_point,
            runner: CairoNativeRunner::new(),
            sierra_program: None,
            params,
            entry_point_id: None,
        }
    }

    /// Init the fuzzer
    /// - Compile Cairo code to Sierra
    /// - Find the entry id
    /// - Init the runner
    pub fn init(&mut self) -> Result<(), String> {
        self.convert_and_store_cairo_to_sierra()?;
        self.entry_point_id = Some(self.find_entry_point_id());
        self.runner.init(&self.entry_point_id, &self.sierra_program)
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
                        /// Map param_type to an `ArgumentType`
                        /// For now we only handle felt252
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

    /// Run the fuzzer
    /// We just use an infinite loop for now
    pub fn fuzz(&mut self) -> Result<(), String> {
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
