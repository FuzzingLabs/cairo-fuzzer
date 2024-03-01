use crate::fuzzer::coverage::Coverage;
use crate::fuzzer::error::Error;
use crate::json_helper::json_parser;
use crate::mutator::types::Type;
use starknet_rs::CasmContractClass;

pub trait Runner {
    /// Runs the selected target
    fn execute(&mut self, inputs: Vec<Type>) -> Result<Option<Coverage>, (Coverage, Error)>;
    /// Returns the target parameters
    fn get_target_parameters(&self) -> Vec<Type>;
    /// Returns the name of the targeted module
    fn get_target_module(&self) -> String;
    /// Returns the name of the targeted function
    fn get_target_function(&self) -> String;
    /// Returns the max coverage
    fn get_max_coverage(&self) -> usize;
    fn get_contract_class(&self) -> CasmContractClass;
    fn get_function(&self) -> json_parser::Function;
}
