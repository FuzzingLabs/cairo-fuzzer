use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::Program;
use cairo_native::context::NativeContext;
use cairo_native::execution_result::ContractExecutionResult;
use cairo_native::executor::JitNativeExecutor;
use cairo_native::module::NativeModule;
use starknet_types_core::felt::Felt;

use crate::runner::syscall_handler::SyscallHandler;

// Create a JIT Native Executor
pub fn create_executor<'a>(native_program: NativeModule<'a>) -> JitNativeExecutor<'a> {
    JitNativeExecutor::from_native_module(native_program, Default::default())
}

/// Compile a Sierra program into a MLIR module
pub fn compile_sierra_program<'a>(
    native_context: &'a NativeContext,
    sierra_program: &'a Program,
) -> Result<NativeModule<'a>, String> {
    native_context
        .compile(sierra_program, false)
        .map_err(|e| e.to_string())
}

/// Execute a program with arbitraty entrypoint & parameters
pub fn run_program(
    executor: &JitNativeExecutor,
    entry_point_id: &FunctionId,
    params: &Vec<Felt>,
) -> Result<ContractExecutionResult, String> {
    executor
        .invoke_contract_dynamic(entry_point_id, params, Some(u128::MAX), SyscallHandler)
        .map_err(|e| e.to_string())
}
