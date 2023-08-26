#[allow(dead_code)]
#[allow(unused_variables)]
/* use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
 */
use cairo_rs::types::program::Program;
/* use cairo_rs::types::relocatable::MaybeRelocatable;
use cairo_rs::vm::runners::cairo_runner::CairoRunner;
use cairo_rs::vm::vm_core::VirtualMachine; */

use felt::Felt252;

use super::runner::Runner;
#[derive(Clone)]
pub struct RunnerCairo {
    program: Program,
}

impl RunnerCairo {
    pub fn new(program: &Program) -> Self {
        return RunnerCairo {
            program: program.clone(),
        };
    }
}

impl Runner for RunnerCairo {
    fn runner(
        self,
        func_name: usize,
        data: &Vec<Felt252>,
    ) -> Result<Option<Vec<(u32, u32)>>, String> {
        /*         // Init the cairo_runner, the VM and the hint_processor
        let mut cairo_runner = CairoRunner::new(&self.program, "small", false)
            .expect("Failed to init the CairoRunner");
        let mut vm = VirtualMachine::new(true);
        let mut hint_processor = BuiltinHintProcessor::new_empty();

        // Set the entrypoint which is the function the user want to fuzz
        let entrypoint = match self
            .program
            .get_identifier(&format!("__main__.{}", &func_name))
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
        let entrypoint_selector = MaybeRelocatable::from(Felt252::new(entrypoint)); // entry point selector => ne sert a rien
                                                                                    // This is used in case of implicit argument
        let value_one = MaybeRelocatable::from((2, 0));
        args.push(entrypoint_selector);
        args.push(value_one);

        // Create a buffer where every u8 in data, will be used to create a MaybeRelocatable
        let buf: Vec<MaybeRelocatable> = data
            .as_slice()
            .iter()
            .map(|x| MaybeRelocatable::from(x))
            .collect();
        // Each u8 of the data will be an argument to the function
        for val in buf {
            args.push(val)
        }
        // This function is a wrapper Fuzzinglabs made to pass the vector of MaybeRelocatable easily
        match cairo_runner.run_from_entrypoint_fuzz(
            entrypoint,
            args,
            true,
            &mut vm,
            &mut hint_processor,
        ) {
            Ok(()) => (),
            Err(e) => return Err(e.to_string()),
        };
        cairo_runner
            .relocate(&mut vm, false)
            .expect("Failed to relocate VM");
        let trace = vm.get_trace();
        let mut ret = Vec::<(u32, u32)>::new();
        for i in trace {
            ret.push((
                i.pc.try_into()
                    .expect("Failed to transform offset into u32"),
                i.fp.try_into()
                    .expect("Failed to transform offset into u32"),
            ))
        } */
        let ret = vec![];
        return Ok(Some(ret));
    }
}
