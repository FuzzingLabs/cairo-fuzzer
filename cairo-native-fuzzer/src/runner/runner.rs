use std::sync::Arc;

use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::Program;
use cairo_native::execution_result::ExecutionResult;
use cairo_native::module::NativeModule;
use cairo_native::{context::NativeContext, executor::JitNativeExecutor, Value};

/// Cairo Runner that uses Cairo Native  
pub struct CairoNativeRunner {
    native_context: NativeContext,
    sierra_program: Option<Arc<Program>>,
    entry_point_id: Option<FunctionId>,
}

/// Compile the sierra program into a MLIR module
fn compile_sierra_program<'a>(
    native_context: &'a NativeContext,
    sierra_program: &'a Program,
) -> Result<NativeModule<'a>, String> {
    native_context
        .compile(sierra_program, false)
        .map_err(|e| e.to_string())
}

// Create the Native Executor (with JIT)
fn create_executor<'a>(native_program: NativeModule<'a>) -> JitNativeExecutor<'a> {
    JitNativeExecutor::from_native_module(native_program, Default::default())
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

    /// Initialize the runner
    /// 1 - Load the sierra_program instance variable
    /// 2 - Load the entry point id in an instance variable
    pub fn init(
        &mut self,
        entry_point_id: &Option<FunctionId>,
        sierra_program: &Option<Arc<Program>>,
    ) -> Result<(), String> {
        self.sierra_program = sierra_program.clone();
        self.entry_point_id = entry_point_id.clone();

        Ok(())
    }

    // Run the program using Cairo Native
    #[inline]
    pub fn run_program(&mut self, params: &[Value]) -> Result<ExecutionResult, String> {
        // Compile the Sierra program into a MLIR module
        let native_program = compile_sierra_program(
            &self.native_context,
            self.sierra_program
                .as_ref()
                .ok_or("Sierra program not available")?,
        )?;

        // Instantiate the executor
        let native_executor = create_executor(native_program);

        // Execute the program
        native_executor
            .invoke_dynamic(self.entry_point_id.as_ref().unwrap(), params, None)
            .map_err(|e| e.to_string())
    }
}
