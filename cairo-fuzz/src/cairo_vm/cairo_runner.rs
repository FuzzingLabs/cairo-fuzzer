use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::types::program::Program;
use cairo_rs::types::relocatable::MaybeRelocatable;
use cairo_rs::types::relocatable::Relocatable;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;
use num_bigint::BigInt;
use num_bigint::Sign;

pub fn runner(
    json: &String,
    func_name: String,
    data: &Vec<u8>,
) -> Result<Option<Vec<(Relocatable, Relocatable)>>, ()> {
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
    let value_zero = MaybeRelocatable::from(Into::<BigInt>::into(entrypoint)); // entry point selector => ne sert a rien
    let value_one = MaybeRelocatable::from((2, 0)); // output_ptr =>
    args.push(value_zero);
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

    //println!("args_num here ==> {}", args_num);
    match cairo_runner.run_from_entrypoint_fuzz(
        entrypoint,
        args,
        //false,
        true,
        //true,
        &mut vm,
        &hint_processor,
    ) {
        Ok(()) => {

            //let rel_table = vm
            //.segments
            //.relocate_segments()
            //.expect("Couldn't relocate after compute effective sizes");
            //cairo_runner.relocate_trace(&mut vm, &rel_table).unwrap();
            cairo_runner.relocate(&mut vm).unwrap();
            //let relocated_trace = cairo_runner.relocated_trace.unwrap();
            //let trace = relocated_trace
            let trace = vm.get_trace().unwrap();
            let mut ret = Vec::<(Relocatable, Relocatable)>::new();
            for i in trace {
                //println!("{:?}", i.fp.clone());
                ret.push((i.fp.clone(), i.pc.clone()));
            }
            let mut stdout = Vec::<u8>::new();
            cairo_runner.write_output(&mut vm, &mut stdout).unwrap();
            /*println!("write output : {:?}", stdout);
            println!("");
            println!(
                "get output : {:?}",
                cairo_runner.get_output(&mut vm).unwrap()
            );
            println!("");*/
            //println!("{:?}", ret);
            return Ok(Some(ret));
            //println!("{:?}", trace);
            /*
                let mut stdout = Vec::<u8>::new();
            cairo_runner.write_output(&mut vm, &mut stdout).unwrap();
            let trace = vm.trace.as_ref().unwrap();
            println!("trace : {:?}", trace);
            println!("write output : {:?}", stdout);
            println!("");
            println!(
                "get output : {:?}",
                cairo_runner.get_output(&mut vm).unwrap()
            );
            println!("");*/
        }
        Err(e) => {
            //let trace = vm.trace.as_ref().unwrap();
            //println!("{:?}", trace);
            println!("{:?}", e);
            panic!("{:?} {:?}", data, e);
        }
    }
}
