use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::Program;
use cairo_native::execution_result::ExecutionResult;
use cairo_native::module::NativeModule;
use cairo_native::{
    context::NativeContext, executor::JitNativeExecutor, utils::cairo_to_sierra, Value,
};
use std::path::Path;
use std::sync::Arc;

pub struct CairoNativeRunner {
    native_context: NativeContext,
    sierra_program: Option<Arc<Program>>,
    entry_point_id: Option<FunctionId>,
}

impl CairoNativeRunner {
    pub fn new() -> Self {
        let native_context = NativeContext::new();
        Self {
            native_context,
            sierra_program: None,
            entry_point_id: None,
        }
    }

    pub fn init(&mut self, program_path: &Path, entry_point: &str) -> Result<(), String> {
        // Convert and store the Sierra program
        self.convert_and_store_cairo_to_sierra(program_path)?;

        // Find and store the entry point ID
        self.entry_point_id = Some(self.find_entry_point_id(entry_point)?);

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

    fn compile_sierra_program(&self) -> Result<NativeModule, String> {
        let sierra_program = self
            .sierra_program
            .as_ref()
            .ok_or("Sierra program not available")?;
        self.native_context
            .compile(sierra_program, false)
            .map_err(|e| e.to_string())
    }

    fn create_executor<'a>(&self, native_program: NativeModule<'a>) -> JitNativeExecutor<'a> {
        JitNativeExecutor::from_native_module(native_program, Default::default())
    }

    // Run the program
    // TODO : Only keep the execution part in this method
    pub fn run_program(&mut self, params: &[Value]) -> Result<ExecutionResult, String> {
        // Compile the Sierra program into a MLIR module
        let native_program = self.compile_sierra_program()?;

        // Instantiate the executor
        let native_executor = self.create_executor(native_program);

        // Execute the program
        native_executor
            .invoke_dynamic(self.entry_point_id.as_ref().unwrap(), params, None)
            .map_err(|e| e.to_string())
    }
}
