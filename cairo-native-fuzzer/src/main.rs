mod fuzzer;
mod runner;

use crate::fuzzer::fuzzer::Fuzzer;

use cairo_native::Value;
use starknet_types_core::felt::Felt;
use std::path::Path;

fn main() {
    let program_path = Path::new("examples/hello.cairo").to_path_buf();
    let entry_point = "hello::hello::greet".to_string();
    let params = vec![Value::Felt252(Felt::from_bytes_be_slice(b"user"))];

    let mut fuzzer = Fuzzer::new(program_path, entry_point, params);

    match fuzzer.init() {
        Ok(()) => match fuzzer.fuzz() {
            Ok(()) => println!("Fuzzing completed successfully."),
            Err(e) => eprintln!("Error during fuzzing: {}", e),
        },
        Err(e) => eprintln!("Error during initialization: {}", e),
    }
}
