use cairo_native::Value;
use std::path::PathBuf;
use crate::runner::runner::CairoNativeRunner;

pub struct Fuzzer {
    program_path: PathBuf,
    entry_point: String,
    runner: CairoNativeRunner,
    params: Vec<Value>,
}

impl Fuzzer {
    pub fn new(program_path: PathBuf, entry_point: String, params: Vec<Value>) -> Self {
        Self {
            program_path,
            entry_point,
            runner: CairoNativeRunner::new(),
            params,
        }
    }

    /// init the fuzzer
    pub fn init(&mut self) -> Result<(), String> {
        self.runner.init(&self.program_path, &self.entry_point)
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
