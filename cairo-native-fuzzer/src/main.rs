mod runner;
use crate::runner::runner::CairoNativeRunner;

use cairo_native::Value;
use starknet_types_core::felt::Felt;
use std::path::Path;

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
