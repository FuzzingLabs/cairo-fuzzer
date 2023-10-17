use cairo_fuzzer::json::json_parser::get_function_from_json;
use cairo_fuzzer::runner::runner::Runner;
use cairo_fuzzer::runner::starknet_runner::RunnerStarknet;
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use felt::Felt252;

use std::fs;

fn main() {
    // Init state
    let casm_file = "../tests1.0/fuzzinglabs_init.casm";
    let sierra_file = "../tests1.0/fuzzinglabs_init.json";
    let casm_content = fs::read_to_string(casm_file).expect("Could not read casm file");
    let sierra_content = fs::read_to_string(sierra_file).expect("Could not read casm file");
    let init_function_name = "init".to_string();

    let function = get_function_from_json(&sierra_content, &init_function_name)
        .expect("Could not get function");
    let contract_class: CasmContractClass =
        serde_json::from_str(&casm_content).expect("could not get contractclass");
    let mut runner = RunnerStarknet::new(&contract_class, function.selector_idx);
    let input: Vec<Felt252> = vec![Felt252::from_bytes_be(&10000000_i64.to_be_bytes())];
    runner = runner.clone().run(&input).unwrap().0;
    let state = runner.get_state();
    println!("================================================");

    // Ready to fuzz other contract
    let casm_file = "../tests1.0/fuzzinglabs_fuzz.casm";
    let sierra_file = "../tests1.0/fuzzinglabs_fuzz.json";
    let casm_content = fs::read_to_string(casm_file).expect("Could not read casm file");
    let sierra_content = fs::read_to_string(sierra_file).expect("Could not read casm file");
    let init_function_name = "storage_test".to_string();

    let function = get_function_from_json(&sierra_content, &init_function_name)
        .expect("Could not get function");
    let contract_class: CasmContractClass =
        serde_json::from_str(&casm_content).expect("could not get contractclass");

    let mut runner = RunnerStarknet::new(&contract_class, function.selector_idx);
    runner = runner.clone().set_state(state.cache);
    let state = runner.clone().get_state();
    let input: Vec<Felt252> = vec![];
    runner = runner.clone().run(&input).unwrap();
}
