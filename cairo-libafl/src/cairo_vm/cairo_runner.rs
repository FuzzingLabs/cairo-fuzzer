use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::types::program::Program;
use cairo_rs::types::relocatable::MaybeRelocatable;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;
use num_bigint::BigInt;
use num_bigint::Sign;
use std::any::Any;

pub fn runner(json: &String, func_name: String, args_num: u64, data: isize){
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
    let value_divide = &MaybeRelocatable::from(Into::<BigInt>::into(data));
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
        Ok(()) => {/* 
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