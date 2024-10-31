use crate::runner::runner::CairoNativeRunner;
use cairo_lang_sierra::program::Program;
use cairo_native::utils::cairo_to_sierra;
use cairo_native::Value;
use std::path::PathBuf;
use std::sync::Arc;

pub struct Fuzzer {
    program_path: PathBuf,
    entry_point: String,
    runner: CairoNativeRunner,
    sierra_program: Option<Arc<Program>>,
    params: Vec<Value>,
}

impl Fuzzer {
    pub fn new(program_path: PathBuf, entry_point: String, params: Vec<Value>) -> Self {
        Self {
            program_path,
            entry_point,
            runner: CairoNativeRunner::new(),
            sierra_program: None,
            params,
        }
    }

    /// Init the fuzzer
    pub fn init(&mut self) -> Result<(), String> {
        self.convert_and_store_cairo_to_sierra()?;
        self.runner.init(&self.entry_point, &self.sierra_program)
    }

    /// Compile the Cairo program to Sierra
    fn convert_and_store_cairo_to_sierra(&mut self) -> Result<(), String> {
        if self.sierra_program.is_none() {
            self.sierra_program = Some(cairo_to_sierra(&self.program_path));
        }
        Ok(())
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
