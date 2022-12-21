use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::types::program::Program;
use cairo_rs::types::relocatable::MaybeRelocatable;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;
use num_bigint::BigInt;
use num_bigint::Sign;
use std::any::Any;
use std::env;
use std::fs;
use pyo3::Python;
use pyo3::PyAny;
use pyo3::ToPyObject;
mod parse_json;
mod cairo_rs_py;
use cairo_rs_py::cairo_runner::PyCairoRunner;
/* use cairo_rs_py;
use cairo_rs_py::cairo_runner as cairo_rs_py_runner;
use cairo_rs_py_runner::PyCairoRunner; */
use crate::parse_json::parse_json;
mod utils;

fn runner(json: &String, func_name: String, args_num: u64, data: isize) {
    println!("\n====> Running function : {}", func_name);
    println!("");
    let program = Program::from_string(json, Some(&func_name)).unwrap();
    let mut cairo_runner = CairoRunner::new(&program, "all", false).unwrap();
    let mut vm = VirtualMachine::new(
        BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
        true,
        Vec::new(),
    );
    let hint_processor = BuiltinHintProcessor::new_empty();

    let entrypoint = match program
        .identifiers
        .get(&format!("__main__.{}", &func_name))
        .unwrap()
        .pc
    {
        Some(value) => value,
        None => return ,
    };

    cairo_runner.initialize_builtins(&mut vm).unwrap();
    cairo_runner.initialize_segments(&mut vm, None);

    //println!("vm segment => {:?}", vm.segments);
    let mut args = Vec::<&dyn Any>::new();
    let value_zero = &MaybeRelocatable::from(Into::<BigInt>::into(entrypoint)); // entry point selector => ne sert a rien
    let value_one = &MaybeRelocatable::from((2,0)); // output_ptr => 
    args.push(value_zero);
    args.push(value_one);
    let value_divide = &MaybeRelocatable::from(Into::<BigInt>::into(5));
    for _i in 0..args_num {
        args.push(value_divide);
    }

    //println!("args_num here ==> {}", args_num);
    match cairo_runner.run_from_entrypoint(
        entrypoint,
        args,
        false,
        true,
        true,
        &mut vm,
        &hint_processor,
    ) {
        Ok(()) => {
            let mut stdout = Vec::<u8>::new();
        cairo_runner.write_output(&mut vm, &mut stdout).unwrap();
        let trace = vm.get_trace().unwrap();
        println!("trace : {:?}", trace);
        println!("write output : {:?}", stdout);
        println!("");
        println!(
            "get output : {:?}",
            cairo_runner.get_output(&mut vm).unwrap()
        );
        println!("");
        return }
        ,
        Err(e) => {
            //let trace = vm.trace.as_ref().unwrap();
            //println!("{:?}", trace);
            println!("{:?}",e);
            panic!("{:?}", e);
        }
    }
}

fn py_runner(json: &String, func_name: String, args_num: u64, data: isize) {
    println!("Running py_runner");
    let mut cairo_runner = PyCairoRunner::new(json.to_string(), Some(func_name.clone()), Some("all".to_string()), false).unwrap();
    //cairo_runner.initialize_function_runner();
    cairo_runner.initialize_segments();
/*     Python::with_gil(|py| {
        cairo_runner
            .run_from_entrypoint(
                py,
                py.eval("0", None, None).unwrap(),
                Vec::<&PyAny>::new().to_object(py),
                None,
                None,
                Some(false),
                None,
                None,
            )
            .unwrap();
    }); */
    let entrypoint: &str = &func_name.clone();
    cairo_runner.cairo_run_py(true, None, None, None, None, Some(entrypoint)).expect("could not run program");
}


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
}*/


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage : cargo run -- <PATH> <PY>");
        return;
    }
    let filename: &String = &args[1];
    let py: bool = &args[2] == "true";
    let functions = parse_json(filename);
    let contents = fs::read_to_string(filename).expect("could not read contract artefact");
    for function in functions {
        if !py {
        runner(&contents, function.name, function.num_args, 5);
        } else {
        py_runner(&contents, function.name, function.num_args, 5);
        }
    }
}
