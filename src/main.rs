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
fn runner() {
    let program =
        Program::from_file(Path::new("/tmp/contract/vuln.json"), Some("main")).unwrap();
    let mut cairo_runner = cairo_runner!(program);
    let mut vm = vm!();
    let hint_processor = BuiltinHintProcessor::new_empty();

    let entrypoint = program
        .identifiers
        .get("__main__.main")
        .unwrap()
        .pc
        .unwrap();

    //vm.accessed_addresses = Some(Vec::new());
    cairo_runner.initialize_function_runner(&mut vm).unwrap();
    //cairo_runner.initialize_builtins(&mut vm).unwrap();
    cairo_runner.initialize_segments(&mut vm, None);
    let _var = cairo_runner.run_from_entrypoint(
            entrypoint,
            vec![&mayberelocatable!(0)],
            false,
            true,
            true,
            &mut vm,
            &hint_processor,
        );
    //write_output(&mut cairo_runner, &mut vm).unwrap();
    //cairo_runner.run_ended = true;
    cairo_runner.end_run(true, false, &mut vm, &hint_processor);
    cairo_runner.read_return_values(&mut vm).unwrap();
}

fn main() {
    runner();
}