use crate::json::json_parser::Function;
use crate::starknet_helper::starknet::StarknetFuzzer;
use rand::Rng;
use std::sync::{Arc, Mutex};
use crate::fuzzer::stats::*;


pub fn starknet_runner(stats: Arc<Mutex<Statistics>>, tx_sequence: &Vec<Function>, starknet_fuzzer: &StarknetFuzzer) ->Result<String, String> {
    let mut rng = rand::thread_rng();
    for func in tx_sequence {
        if func.decorators.contains(&"view".to_string()) {
            println!("call");
            if ! starknet_fuzzer.call_contract(&func.name) {
                let mut stats = stats.lock().expect("Failed to get mutex");
                stats.crashes += 1;
            }
        } else {
            println!("invoke");
            let mut inputs: String = "".to_string();
            for _i in 0..func.num_args {
                let value: u8 = rng.gen();
                inputs += &format!("{} ", value).to_string();
            }
            if !starknet_fuzzer.invoke_contract(&func.name, &inputs) {
                let mut stats = stats.lock().expect("Failed to get mutex");
                stats.crashes += 1;
            }
        }
        let mut stats = stats.lock().expect("Failed to get mutex");
        // Update fuzz case count
        stats.fuzz_cases += 1;
    }
    Ok("good".to_string())
}
