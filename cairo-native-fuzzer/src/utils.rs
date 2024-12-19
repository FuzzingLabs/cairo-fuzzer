use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::Program;
use cairo_lang_sierra::program::StatementIdx;

/// Return the current cairo-native package version
pub fn get_cairo_native_version() -> String {
    // TODO : Automatically parse Cargo.toml
    "0.2.5-rc1".to_string()
}

/// Find and return the function with the given `FunctionId` in the `Program`
pub fn get_function_by_id<'a>(
    program: &'a Program,
    function_id: &FunctionId,
) -> Option<&'a GenFunction<StatementIdx>> {
    program.funcs.iter().find(|f| &f.id == function_id)
}
