use cairo_rs::types::program::Program;
use std::path::Path;
use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::types::relocatable::MaybeRelocatable;
use num_bigint::BigInt;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;
use num_bigint::Sign;
use std::any::Any;
use std::env;
mod parse_json;
use crate::parse_json::parse_json;
mod utils;

fn runner(file_path: &String, func_name: String, args_num: u64) {
    println!("====> Running function : {}", func_name);
    println!("");
    let program =
        Program::from_file(Path::new(&file_path), Some(&func_name)).unwrap();
    let mut cairo_runner = cairo_runner!(program);
    let mut vm = vm!();
    let hint_processor = BuiltinHintProcessor::new_empty();

    let entrypoint = match program
        .identifiers
        .get(&format!("__main__.{}", &func_name))
        .unwrap()
        .pc {
            Some(value) => value,
            None => return,
        };

    cairo_runner.initialize_builtins(&mut vm).unwrap();
    cairo_runner.initialize_segments(&mut vm, None);
    let value = &MaybeRelocatable::from((2,0));
    let mut args = Vec::<&dyn Any>::new();
    args.push(value);
    for _i in 0..args_num {
        args.push(value);
    }
    let _var = cairo_runner.run_from_entrypoint(
            entrypoint,
            args,
            false,
            true,
            true,
            &mut vm,
            &hint_processor,
        );

    let mut stdout = Vec::<u8>::new();
    cairo_runner.write_output(&mut vm, &mut stdout).unwrap();
    println!("write output : {:?}", stdout); 
    println!("");
    println!("get output : {:?}", cairo_runner.get_output(&mut vm).unwrap());
    println!(""); 
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage : cargo run -- <PATH>");
        return;
    }
    let filename: &String = &args[1];
    let functions = parse_json(filename);
    for function in functions {
    runner(filename, function.name, function.num_args);
    }
}