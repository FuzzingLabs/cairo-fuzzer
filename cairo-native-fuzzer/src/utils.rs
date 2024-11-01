use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::Program;
use cairo_lang_sierra::program::StatementIdx;

/// Find and return the function with the given `FunctionId` in the `Program`
pub fn get_function_by_id<'a>(
    program: &'a Program,
    function_id: &FunctionId,
) -> Option<&'a GenFunction<StatementIdx>> {
    program.funcs.iter().find(|f| &f.id == function_id)
}
