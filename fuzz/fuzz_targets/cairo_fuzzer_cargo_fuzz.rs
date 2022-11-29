//#[macro_use]
//extern crate honggfuzz;
#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::types::program::Program;
use cairo_rs::types::relocatable::MaybeRelocatable;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;
//use cargo_libafl_helper::fuzz_target;
use num_bigint::BigInt;
use num_bigint::Sign;
use std::any::Any;
use std::env;
use std::fs;
use std::path::Path;
mod parse_json;
use crate::parse_json::parse_json;
use crate::parse_json::Function;
mod utils;
#[macro_use]
extern crate lazy_static;

fn runner(json: &String, func_name: String, args_num: u64, data: isize) {
    //println!("====> Running function : {}", func_name);
    //println!("");
    let program = Program::from_string(json, Some(&func_name)).unwrap();
    let mut cairo_runner = cairo_runner!(program);
    let mut vm = vm!();
    let hint_processor = BuiltinHintProcessor::new_empty();

    let entrypoint = match program
        .identifiers
        .get(&format!("__main__.{}", &func_name))
        .unwrap()
        .pc
    {
        Some(value) => value,
        None => return,
    };

    cairo_runner.initialize_builtins(&mut vm).unwrap();
    cairo_runner.initialize_segments(&mut vm, None);
    let value = &MaybeRelocatable::from((data, 0));
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
    /*println!("write output : {:?}", stdout);
    println!("");
    println!(
        "get output : {:?}",
        cairo_runner.get_output(&mut vm).unwrap()
    );
    println!("");*/
}

lazy_static! {
    static ref FUNCTIONS: Vec<Function> = parse_json(&"json/vuln.json".to_string());
    static ref CONTENTS: std::string::String = fs::read_to_string(&"json/vuln.json".to_string())
        .expect("Should have been able to read the file");
}

fuzz_target!(|data: isize| {
    for function in FUNCTIONS.clone() {
        runner(&CONTENTS, function.name, function.num_args, 2);
    }
});

/*
fn main() {
    let functions = parse_json(&"json/vuln.json".to_string());
    let contents = fs::read_to_string(&"json/vuln.json".to_string())
        .expect("Should have been able to read the file");
    /*for function in functions {
        runner(&contents, function.name, function.num_args, 2);
    }*/
    loop {
        fuzz!(|data: isize| {
            for function in functions.clone() {
                runner(&contents, function.name, function.num_args, data);
            }
        });
    }
} */

/*
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
*/
