use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::types::program::Program;
use cairo_rs::types::relocatable::MaybeRelocatable;
use cairo_rs::types::relocatable::Relocatable;
use cairo_rs::vm::errors::vm_errors::VirtualMachineError;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;

use num_bigint::BigInt;
use num_bigint::Sign;

pub fn runner(
    json: &String,
    func_name: &String,
    data: &Vec<u8>,
) -> Result<Option<Vec<(Relocatable, Relocatable)>>, VirtualMachineError> {
    // Init program from the json content
    let program = Program::from_string(json, Some(&func_name)).unwrap();
    // Init the cairo_runner, the VM and the hint_processor
    let mut cairo_runner = CairoRunner::new(&program, "all", false).unwrap();
    let mut vm = VirtualMachine::new(
        BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
        true,
        Vec::new(),
    );
    let hint_processor = BuiltinHintProcessor::new_empty();

    // Set the entrypoint which is the function the user want to fuzz
    let entrypoint = match program
        .identifiers
        .get(&format!("__main__.{}", &func_name))
        .unwrap()
        .pc
    {
        Some(value) => value,
        None => return Ok(None),
    };

    // Init builtins and segments
    cairo_runner.initialize_builtins(&mut vm).unwrap();
    cairo_runner.initialize_segments(&mut vm, None);

    // Init the vector of arguments
    let mut args = Vec::<MaybeRelocatable>::new();
    // Set the entrypoint selector
    let entrypoint_selector = MaybeRelocatable::from(Into::<BigInt>::into(entrypoint)); // entry point selector => ne sert a rien
                                                                                        // This is used in case of implicit argument
    let value_one = MaybeRelocatable::from((2, 0));
    args.push(entrypoint_selector);
    args.push(value_one);

    // Create a buffer where every u8 in data, will be used to create a MaybeRelocatable
    let buf: Vec<MaybeRelocatable> = data
        .as_slice()
        .iter()
        .map(|x| MaybeRelocatable::from(Into::<BigInt>::into(*x)))
        .collect();
    // Each u8 of the data will be an argument to the function
    for val in buf {
        args.push(val)
    }
    // This function is a wrapper Fuzzinglabs made to pass the vector of MaybeRelocatable easily
    match cairo_runner.run_from_entrypoint_fuzz(entrypoint, args, true, &mut vm, &hint_processor) {
        Ok(()) => {
            cairo_runner.relocate(&mut vm).unwrap();
            let trace = vm.get_trace().unwrap();
            let mut ret = Vec::<(Relocatable, Relocatable)>::new();
            for i in trace {
                ret.push((i.fp.clone(), i.pc.clone()));
            }
            let mut stdout = Vec::<u8>::new();
            cairo_runner.write_output(&mut vm, &mut stdout).unwrap();

            return Ok(Some(ret));
        }
        Err(e) => {
            return Err(e);
        }
    }
}
