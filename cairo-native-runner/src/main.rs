use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::Program;
use cairo_native::execution_result::ExecutionResult;
use cairo_native::module::NativeModule;
use cairo_native::{
    context::NativeContext, executor::JitNativeExecutor, utils::cairo_to_sierra, Value,
};
use starknet_types_core::felt::Felt;
use std::path::Path;
use std::sync::Arc;

pub struct CairoNativeRunner<'a> {
    native_context: NativeContext,
    sierra_program: Option<Arc<Program>>,
    entry_point_id: Option<FunctionId>,
    native_module: Option<Box<NativeModule<'a>>>,
}

impl<'a> CairoNativeRunner<'a> {
    pub fn new() -> Self {
        let native_context = NativeContext::new();
        Self {
            native_context,
            sierra_program: None,
            entry_point_id: None,
            native_module: None,
        }
    }

    /// Initialize the runner
    pub fn init(&mut self, program_path: &Path, entry_point: &str) -> Result<(), String> {
        // Convert and store the Sierra program
        self.convert_and_store_cairo_to_sierra(program_path)?;

        // Find and store the entry point ID
        self.entry_point_id = Some(self.find_entry_point_id(entry_point)?);

        // Compile the Sierra program into a MLIR module
        let native_module = self.compile_sierra_program()?;

        // Assign the compiled module to self.native_module
        self.native_module = Some(native_module);

        Ok(())
    }

    fn convert_and_store_cairo_to_sierra(&mut self, program_path: &Path) -> Result<(), String> {
        if self.sierra_program.is_none() {
            self.sierra_program = Some(cairo_to_sierra(program_path));
        }
        Ok(())
    }

    fn find_entry_point_id(&self, entry_point: &str) -> Result<FunctionId, String> {
        let sierra_program = self
            .sierra_program
            .as_ref()
            .ok_or("Sierra program not available")?;
        cairo_native::utils::find_function_id(sierra_program, entry_point)
            .ok_or_else(|| format!("Entry point '{}' not found", entry_point))
            .cloned()
    }

    fn compile_sierra_program(&self) -> Result<Box<NativeModule<'a>>, String> {
        let sierra_program = self
            .sierra_program
            .as_ref()
            .ok_or("Sierra program not available")?;
        let native_module = self
            .native_context
            .compile(sierra_program, false)
            .map_err(|e| e.to_string())?;
        Ok(Box::new(native_module))
    }

    fn create_executor(native_program: Box<NativeModule<'a>>) -> JitNativeExecutor<'a> {
        JitNativeExecutor::from_native_module(*native_program, Default::default())
    }

    /// Run the program
    /// TODO : keep only the execution part in the method
    pub fn run_program(&mut self, params: &[Value]) -> Result<ExecutionResult, String> {
        // Compile the Sierra program into a MLIR module
        let native_program = self.compile_sierra_program()?;

        // Instantiate the executor
        let native_executor = Self::create_executor(native_program);

        // Execute the program
        native_executor
            .invoke_dynamic(self.entry_point_id.as_ref().unwrap(), params, None)
            .map_err(|e| e.to_string())
    }
}

fn main() {
    let program_path = Path::new("examples/hello.cairo");
    let entry_point = "hello::hello::greet";
    let params = &[Value::Felt252(Felt::from_bytes_be_slice(b"user"))];

    let mut runner = CairoNativeRunner::new();

    match runner.init(program_path, entry_point) {
        Ok(()) => match runner.run_program(params) {
            Ok(result) => {
                println!("Cairo program was compiled and executed successfully.");
                println!("{:?}", result);
            }
            Err(e) => eprintln!("Error during execution: {}", e),
        },
        Err(e) => eprintln!("Error during initialization: {}", e),
    }
}
