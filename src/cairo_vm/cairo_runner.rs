use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_rs::types::program::Program;
use cairo_rs::types::relocatable::MaybeRelocatable;
use cairo_rs::types::relocatable::Relocatable;
use cairo_rs::vm::errors::vm_errors::VirtualMachineError;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine;
use cairo_rs_py::cairo_runner::PyCairoRunner;
use num_bigint::BigInt;
use num_bigint::Sign;
use pyo3::PyAny;
use pyo3::ToPyObject;
use pyo3::marker::Python;
use pyo3::types::PyFloat;
use pyo3::types::PyInt;

pub fn runner(
    json: &String,
    func_name: &String,
    data: &Vec<u8>,
) -> Result<Option<Vec<(Relocatable, Relocatable)>>, VirtualMachineError> {
    // Init program from the json content
    let program =
        Program::from_string(json, Some(&func_name)).expect("Failed to deserialize Program");
    // Init the cairo_runner, the VM and the hint_processor
    let mut cairo_runner =
        CairoRunner::new(&program, "all", false).expect("Failed to init the CairoRunner");
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
        .expect("Failed to initialize entrypoint")
        .pc
    {
        Some(value) => value,
        None => return Ok(None),
    };

    // Init builtins and segments
    cairo_runner
        .initialize_builtins(&mut vm)
        .expect("Failed to initialize builtins");
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
    cairo_runner.run_from_entrypoint_fuzz(entrypoint, args, true, &mut vm, &hint_processor)?;
    cairo_runner
        .relocate(&mut vm)
        .expect("Failed to relocate VM");
    let trace = vm
        .get_trace()
        .expect("Failed to get running trace from the VM");
    let mut ret = Vec::<(Relocatable, Relocatable)>::new();
    for i in trace {
        ret.push((i.fp.clone(), i.pc.clone()));
    }
    let mut stdout = Vec::<u8>::new();
    cairo_runner
        .write_output(&mut vm, &mut stdout)
        .expect("Failed to get running output from the VM");

    return Ok(Some(ret));
}


pub fn py_runner(json: &String,
    func_name: &String,
    data: &Vec<u8>,) -> Result<Option<Vec<(Relocatable, Relocatable)>>, VirtualMachineError> {
    let mut runner = PyCairoRunner::new(
        json.clone(),
        Some(func_name.clone()),
        Some("plain".to_string()),
        false,
    )
    .unwrap();
    runner.initialize_segments();
    let mut ret = Vec::<(Relocatable, Relocatable)>::new();
    Python::with_gil(|py| {
        let mut args = data.to_object(py);
        match runner
            .run_from_entrypoint(
                py,
                py.eval("1", None, None).unwrap(),
                args,
                None,
                None,
                Some(false),
                None,
                None,
            )
            {
                Ok(_val) => println!("good"),
                Err(e) => println!("bad {:?}", e),
            }
            //runner.write_output();
    });
    return Ok(Some(ret));
}