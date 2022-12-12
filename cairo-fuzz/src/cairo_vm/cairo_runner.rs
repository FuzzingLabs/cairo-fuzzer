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
        None => return Ok(None),
    };

    cairo_runner.initialize_builtins(&mut vm).unwrap();
    cairo_runner.initialize_segments(&mut vm, None);

    let mut args = Vec::<MaybeRelocatable>::new();
    let entrypoint_selector = MaybeRelocatable::from(Into::<BigInt>::into(entrypoint)); // entry point selector => ne sert a rien
    let value_one = MaybeRelocatable::from((2, 0)); // output_ptr =>
    args.push(entrypoint_selector);
    args.push(value_one);
    let target = data;
    let buf: Vec<MaybeRelocatable> = target
        .as_slice()
        .iter()
        .map(|x| MaybeRelocatable::from(Into::<BigInt>::into(*x)))
        .collect();
    for val in buf {
        args.push(val)
    }

    match cairo_runner.run_from_entrypoint_fuzz(entrypoint, args, true, &mut vm, &hint_processor) {
        Ok(()) => {
            cairo_runner.relocate(&mut vm).unwrap();
            let trace = vm.get_trace().unwrap();
            let mut ret = Vec::<(Relocatable, Relocatable)>::new();
            for i in trace {
                //println!("{:?}", i.fp.clone());
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
