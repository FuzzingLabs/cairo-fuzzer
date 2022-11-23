use cairo_rs::types::program::Program;
use std::path::Path;
use cairo_rs::cairo_run::write_output;
use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::types::relocatable::MaybeRelocatable;
use num_bigint::BigInt;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;
use num_bigint::Sign;

macro_rules! bigint {
    ($val : expr) => {
        Into::<BigInt>::into($val)
    };
}

macro_rules! mayberelocatable {
    ($val1 : expr, $val2 : expr) => {
        MaybeRelocatable::from(($val1, $val2))
    };
    ($val1 : expr) => {
        MaybeRelocatable::from((bigint!($val1)))
    };
}
macro_rules! cairo_runner {
    ($program:expr) => {
        CairoRunner::new(&$program, "all", false).unwrap()
    };
    ($program:expr, $layout:expr) => {
        CairoRunner::new(&$program, $layout, false).unwrap()
    };
    ($program:expr, $layout:expr, $proof_mode:expr) => {
        CairoRunner::new(&$program, $layout, $proof_mode).unwrap()
    };
    ($program:expr, $layout:expr, $proof_mode:expr) => {
        CairoRunner::new(&program, $layout.to_string(), proof_mode).unwrap()
    };
}
macro_rules! vm {
    () => {{
        VirtualMachine::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        )
    }};

    ($use_trace:expr) => {{
        VirtualMachine::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            $use_trace,
        )
    }};
}

fn runner(file_path: String, func_name: String) {
    let program =
        Program::from_file(Path::new(&file_path), Some(&func_name)).unwrap();
    let mut cairo_runner = cairo_runner!(program);
    let mut vm = vm!();
    let hint_processor = BuiltinHintProcessor::new_empty();

    let entrypoint = program
        .identifiers
        .get(&format!("__main__.{}", &func_name))
        .unwrap()
        .pc
        .unwrap();

    //cairo_runner.initialize(&mut vm).unwrap();
    cairo_runner.initialize_function_runner(&mut vm).unwrap();
    //vm.accessed_addresses = Some(Vec::new());
    //cairo_runner.initialize_function_runner(&mut vm).unwrap();
    cairo_runner.initialize_builtins(&mut vm).unwrap();
    //cairo_runner.initialize_segments(&mut vm, None);
    let _var = cairo_runner.run_from_entrypoint(
            entrypoint,
            vec![&mayberelocatable!(123456)],
            false,
            true,
            true,
            &mut vm,
            &hint_processor,
        );

    let mut stdout = Vec::<u8>::new();
    cairo_runner.write_output(&mut vm, &mut stdout).unwrap();
    //assert_eq!(String::from_utf8(stdout), Ok(String::from("1\n17\n")));
    println!("{:?}", stdout); 


    //write_output(&mut cairo_runner, &mut vm).unwrap();
    //cairo_runner.run_ended = true;
    //cairo_runner.end_run(false, true, &mut vm, &hint_processor);
    //cairo_runner.finalize_segments(&mut vm).unwrap();
    println!("{:?}", cairo_runner.get_output(&mut vm).unwrap()); 
    //cairo_runner.read_return_values(&mut vm).unwrap();
}

fn main() {


/*
%builtins output

from starkware.cairo.common.serialize import serialize_word

func return_10() -> (res : felt):
    let res = 10
    return (res)
end

func main{output_ptr : felt*}():
    
    let (value) = return_10()

    serialize_word(value)

    return ()
end

program=json/cairo_function_return_to_variable.json
*/

    // get return value
    let args = vec![&mayberelocatable!(123456)];
    runner("json/cairo_function_return_to_variable.json".to_string(), "return_10".to_string());

    println!("======");
    // get output_ptr
    runner("json/cairo_function_return_to_variable.json".to_string(), "main".to_string());
}