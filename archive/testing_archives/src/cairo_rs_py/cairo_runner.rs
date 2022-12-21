use crate::{
    cairo_rs_py::memory::PyMemory,
    cairo_rs_py::relocatable::{PyMaybeRelocatable, PyRelocatable},
    cairo_rs_py::utils::to_py_error,
    cairo_rs_py::vm_core::PyVM,
};
use cairo_rs::{
    bigint,
    cairo_run::write_output,
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
    serde::deserialize_program::Member,
    types::{
        program::Program,
        relocatable::{MaybeRelocatable, Relocatable},
    },
    vm::{
        errors::{
            cairo_run_errors::CairoRunError, runner_errors::RunnerError, trace_errors::TraceError,
        },
        runners::cairo_runner::{CairoRunner, ExecutionResources},
        security::verify_secure_runner,
    },
};
use num_bigint::BigInt;
use pyo3::{
    exceptions::{PyNotImplementedError, PyTypeError, PyValueError},
    prelude::*,
    types::PyIterator,
};
use std::{any::Any, borrow::BorrowMut, collections::HashMap, iter::zip, path::PathBuf, rc::Rc};

const MEMORY_GET_SEGMENT_USED_SIZE_MSG: &str = "Failed to segment used size";
const FAILED_TO_GET_INITIAL_FP: &str = "Failed to get initial segment";

#[pyclass(unsendable)]
#[pyo3(name = "CairoRunner")]
pub struct PyCairoRunner {
    inner: CairoRunner,
    pyvm: PyVM,
    hint_processor: BuiltinHintProcessor,
    hint_locals: HashMap<String, PyObject>,
    struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
    static_locals: Option<HashMap<String, PyObject>>,
}

#[pymethods]
impl PyCairoRunner {
    #[new]
    pub fn new(
        program: String,
        entrypoint: Option<String>,
        layout: Option<String>,
        proof_mode: bool,
    ) -> PyResult<Self> {
        let program =
            Program::from_reader(program.as_bytes(), entrypoint.as_deref()).map_err(to_py_error)?;
        let cairo_runner = CairoRunner::new(
            &program,
            &layout.unwrap_or_else(|| "plain".to_string()),
            proof_mode,
        )
        .map_err(to_py_error)?;

        let struct_types = program
            .identifiers
            .iter()
            .filter_map(|(path, identifier)| match identifier.type_.as_deref() {
                Some("struct") => Some((path.to_string(), identifier.members.clone().unwrap())),
                _ => None,
            })
            .collect();

        Ok(PyCairoRunner {
            inner: cairo_runner,
            pyvm: PyVM::new(program.prime, true, program.error_message_attributes),
            hint_processor: BuiltinHintProcessor::new_empty(),
            hint_locals: HashMap::new(),
            struct_types: Rc::new(struct_types),
            static_locals: None,
        })
    }

    #[pyo3(name = "cairo_run")]
    pub fn cairo_run_py(
        &mut self,
        print_output: bool,
        trace_file: Option<&str>,
        memory_file: Option<&str>,
        hint_locals: Option<HashMap<String, PyObject>>,
        static_locals: Option<HashMap<String, PyObject>>,
        entrypoint: Option<&str>,
    ) -> PyResult<()> {
        if let Some(entrypoint) = entrypoint {
            self.inner
                .borrow_mut()
                .set_entrypoint(Some(entrypoint))
                .map_err(to_py_error)?;
        }

        let end = self.initialize()?;
        if let Some(locals) = hint_locals {
            self.hint_locals = locals
        }

        self.static_locals = static_locals;

        if trace_file.is_none() {
            (*self.pyvm.vm).borrow_mut().disable_trace();
        }
        self.run_until_pc(&end)?;

        self.inner
            .end_run(
                false,
                false,
                &mut (*self.pyvm.vm).borrow_mut(),
                &self.hint_processor,
            )
            .map_err(to_py_error)?;

        (*self.pyvm.vm)
            .borrow_mut()
            .verify_auto_deductions()
            .map_err(to_py_error)?;

        self.relocate()?;

        if print_output {
            self.write_output()?;
        }

        if let Some(trace_path) = trace_file {
            let trace_path = PathBuf::from(trace_path);
            let relocated_trace = self
                .inner
                .relocated_trace
                .as_ref()
                .ok_or(CairoRunError::Trace(TraceError::TraceNotEnabled))
                .map_err(to_py_error)?;

            match cairo_rs::cairo_run::write_binary_trace(relocated_trace, &trace_path) {
                Ok(()) => (),
                Err(_e) => {
                    return Err(CairoRunError::Runner(RunnerError::WriteFail)).map_err(to_py_error)
                }
            }
        }

        if let Some(memory_path) = memory_file {
            let memory_path = PathBuf::from(memory_path);
            cairo_rs::cairo_run::write_binary_memory(&self.inner.relocated_memory, &memory_path)
                .map_err(|_| to_py_error(CairoRunError::Runner(RunnerError::WriteFail)))?;
        }

        Ok(())
    }

    pub fn initialize(&mut self) -> PyResult<PyRelocatable> {
        self.inner
            .initialize(&mut (*self.pyvm.vm).borrow_mut())
            .map(PyRelocatable::from)
            .map_err(to_py_error)
    }

    pub fn initialize_segments(&mut self) {
        self.inner
            .initialize_segments(&mut (*self.pyvm.vm).borrow_mut(), None)
    }

    pub fn run_until_pc(&mut self, address: &PyRelocatable) -> PyResult<()> {
        let references = self.inner.get_reference_list();
        let hint_data_dictionary = self
            .inner
            .get_hint_data_dictionary(&references, &self.hint_processor)
            .map_err(to_py_error)?;

        let address = Into::<Relocatable>::into(address);
        let constants = self.inner.get_constants().clone();
        while self.pyvm.vm.borrow().get_pc() != &address {
            self.pyvm
                .step(
                    &self.hint_processor,
                    &mut self.hint_locals,
                    &mut self.inner.exec_scopes,
                    &hint_data_dictionary,
                    Rc::clone(&self.struct_types),
                    &constants,
                    self.static_locals.as_ref(),
                )
                .map_err(to_py_error)?;
        }
        Ok(())
    }

    pub fn mark_as_accessed(&mut self, address: PyRelocatable, size: usize) -> PyResult<()> {
        self.inner
            .mark_as_accessed((&address).into(), size)
            .map_err(to_py_error)
    }

    pub fn relocate(&mut self) -> PyResult<()> {
        self.inner
            .relocate(&mut (*self.pyvm.vm).borrow_mut())
            .map_err(to_py_error)
    }

    pub fn get_output(&mut self) -> PyResult<String> {
        self.inner
            .get_output(&mut (*self.pyvm.vm).borrow_mut())
            .map_err(to_py_error)
    }

    pub fn write_output(&mut self) -> PyResult<()> {
        write_output(&mut self.inner, &mut (*self.pyvm.vm).borrow_mut()).map_err(to_py_error)
    }

    pub fn add_segment(&self) -> PyRelocatable {
        (*self.pyvm.vm).borrow_mut().add_memory_segment().into()
    }

    pub fn get_program_builtins_initial_stack(&self, py: Python) -> PyObject {
        (*self.pyvm.vm)
            .borrow_mut()
            .get_builtin_runners()
            .iter()
            .filter(|(builtin_name, _builtin_runner)| {
                self.inner.get_program_builtins().contains(builtin_name)
            })
            .flat_map(|(_builtin_name, builtin_runner)| {
                builtin_runner
                    .initial_stack()
                    .into_iter()
                    .map(Into::<PyMaybeRelocatable>::into)
                    .collect::<Vec<PyMaybeRelocatable>>()
            })
            .collect::<Vec<PyMaybeRelocatable>>()
            .to_object(py)
    }

    pub fn get_builtins_initial_stack(&self, py: Python) -> PyObject {
        (*self.pyvm.vm)
            .borrow_mut()
            .get_builtin_runners()
            .iter()
            .map(|(_builtin_name, builtin_runner)| {
                builtin_runner
                    .initial_stack()
                    .into_iter()
                    .map(Into::<PyMaybeRelocatable>::into)
                    .collect::<Vec<PyMaybeRelocatable>>()
            })
            .collect::<Vec<Vec<PyMaybeRelocatable>>>()
            .to_object(py)
    }

    pub fn get_builtins_final_stack(&self, stack_ptr: PyRelocatable) -> PyResult<PyRelocatable> {
        let mut stack_ptr = Relocatable::from(&stack_ptr);
        let mut stop_ptrs = Vec::new();
        let mut stop_ptr;

        for (_, runner) in self
            .pyvm
            .vm
            .borrow()
            .get_builtin_runners()
            .iter()
            .rev()
            .filter(|(builtin_name, _builtin_runner)| {
                self.inner.get_program_builtins().contains(builtin_name)
            })
        {
            (stack_ptr, stop_ptr) = runner
                .final_stack(&self.pyvm.vm.borrow(), stack_ptr)
                .map_err(to_py_error)?;
            stop_ptrs.push(stop_ptr);
        }

        for ((_, runner), stop_ptr) in zip(
            (*self.pyvm.vm).borrow_mut().get_builtin_runners_as_mut(),
            stop_ptrs,
        ) {
            runner.set_stop_ptr(stop_ptr);
        }

        Ok(stack_ptr.into())
    }

    pub fn get_execution_resources(&self) -> PyResult<PyExecutionResources> {
        self.inner
            .get_execution_resources(&self.pyvm.vm.borrow())
            .map(PyExecutionResources)
            .map_err(to_py_error)
    }

    pub fn get_ap(&self) -> PyResult<PyRelocatable> {
        Ok(PyRelocatable::from(self.pyvm.vm.borrow().get_ap()))
    }

    #[getter]
    pub fn initial_fp(&self) -> PyResult<PyRelocatable> {
        Ok(PyRelocatable::from(
            self.inner
                .get_initial_fp()
                .ok_or_else(|| PyTypeError::new_err(FAILED_TO_GET_INITIAL_FP))?,
        ))
    }

    pub fn get_return_values(&self, n_ret: usize, py: Python) -> PyResult<PyObject> {
        let return_values = self
            .pyvm
            .get_vm()
            .borrow()
            .get_return_values(n_ret)
            .map_err(|err| pyo3::exceptions::PyException::new_err(format!("{err}")))?
            .into_iter()
            .map(|maybe_reloc| maybe_reloc.into())
            .collect::<Vec<PyMaybeRelocatable>>()
            .to_object(py);
        Ok(return_values)
    }

    pub fn get_segment_used_size(&self, index: usize, py: Python) -> PyResult<PyObject> {
        Ok(self
            .pyvm
            .vm
            .borrow()
            .get_segment_used_size(index)
            .ok_or_else(|| PyTypeError::new_err(MEMORY_GET_SEGMENT_USED_SIZE_MSG))?
            .to_object(py))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_from_entrypoint(
        &mut self,
        py: Python,
        entrypoint: &PyAny,
        args: Py<PyAny>,
        hint_locals: Option<HashMap<String, PyObject>>,
        static_locals: Option<HashMap<String, PyObject>>,
        typed_args: Option<bool>,
        verify_secure: Option<bool>,
        apply_modulo_to_args: Option<bool>,
    ) -> PyResult<()> {
        enum Either {
            MaybeRelocatable(MaybeRelocatable),
            VecMaybeRelocatable(Vec<MaybeRelocatable>),
        }

        impl Either {
            pub fn as_any(&self) -> &dyn Any {
                match self {
                    Self::MaybeRelocatable(x) => x as &dyn Any,
                    Self::VecMaybeRelocatable(x) => x as &dyn Any,
                }
            }
        }

        if let Some(locals) = hint_locals {
            self.hint_locals = locals
        }

        self.static_locals = static_locals;
        let apply_modulo_to_args = apply_modulo_to_args.unwrap_or(true);

        let entrypoint = if let Ok(x) = entrypoint.extract::<usize>() {
            x
        } else if entrypoint.extract::<String>().is_ok() {
            return Err(PyNotImplementedError::new_err(()));
        } else {
            return Err(PyTypeError::new_err("entrypoint must be int or str"));
        };

        let stack = if typed_args.unwrap_or(false) {
            let args = self
                .gen_typed_args(py, args.to_object(py))
                .map_err(to_py_error)?;
            let mut stack = Vec::new();
            for arg in args.extract::<Vec<&PyAny>>(py)? {
                let arg: MaybeRelocatable = arg.extract::<PyMaybeRelocatable>()?.into();
                if apply_modulo_to_args {
                    let arg = arg
                        .mod_floor(self.pyvm.vm.borrow().get_prime())
                        .map_err(to_py_error)?;
                    stack.push(arg)
                } else {
                    stack.push(arg)
                }
            }
            stack
        } else {
            let mut processed_args = Vec::new();
            for arg in args.extract::<Vec<&PyAny>>(py)? {
                let arg_box = if let Ok(x) = arg.extract::<PyMaybeRelocatable>() {
                    Either::MaybeRelocatable(x.into())
                } else if let Ok(x) = arg.extract::<Vec<PyMaybeRelocatable>>() {
                    Either::VecMaybeRelocatable(x.into_iter().map(|x| x.into()).collect())
                } else {
                    return Err(PyTypeError::new_err("Argument has unsupported type."));
                };

                processed_args.push(arg_box);
            }
            let processed_args: Vec<&dyn Any> = processed_args.iter().map(|x| x.as_any()).collect();
            let mut stack = Vec::new();
            for arg in processed_args {
                let prime = match apply_modulo_to_args {
                    true => Some(self.pyvm.vm.borrow().get_prime().clone()),
                    false => None,
                };

                stack.push(
                    (*self.pyvm.vm)
                        .borrow_mut()
                        .gen_arg(arg, prime.as_ref())
                        .map_err(to_py_error)?,
                );
            }

            stack
        };

        let return_fp = MaybeRelocatable::from(bigint!(0));

        let end = self
            .inner
            .initialize_function_entrypoint(
                &mut (*self.pyvm.vm).borrow_mut(),
                entrypoint,
                stack,
                return_fp,
            )
            .map_err(to_py_error)?;

        self.inner
            .initialize_vm(&mut (*self.pyvm.vm).borrow_mut())
            .map_err(to_py_error)?;

        self.run_until_pc(&PyRelocatable::from(end))?;

        self.inner
            .end_run(
                true,
                false,
                &mut (*self.pyvm.vm).borrow_mut(),
                &self.hint_processor,
            )
            .map_err(to_py_error)?;

        if verify_secure.unwrap_or(true) {
            verify_secure_runner(&self.inner, false, &mut (*self.pyvm.vm).borrow_mut())
                .map_err(to_py_error)?;
        }

        Ok(())
    }

    /// Inserts a value into a memory address given by a Relocatable value.
    pub fn insert(&self, key: &PyRelocatable, value: PyMaybeRelocatable) -> PyResult<()> {
        (*self.pyvm.vm)
            .borrow_mut()
            .insert_value(&key.into(), value)
            .map_err(to_py_error)
    }

    // Initialize all the builtins and segments.
    pub fn initialize_function_runner(&mut self) -> PyResult<()> {
        self.inner
            .initialize_function_runner(&mut (*self.pyvm.vm).borrow_mut())
            .map_err(to_py_error)
    }

    pub fn gen_arg(
        &self,
        py: Python,
        arg: Py<PyAny>,
        apply_modulo_to_args: bool,
    ) -> PyResult<PyObject> {
        Ok(
            PyMaybeRelocatable::from(match PyIterator::from_object(py, &arg) {
                Ok(iterator) => {
                    let segment_ptr = MaybeRelocatable::RelocatableValue(
                        (*self.pyvm.vm).borrow_mut().add_memory_segment(),
                    );
                    self.write_arg(
                        py,
                        segment_ptr.clone().into(),
                        iterator.to_object(py),
                        apply_modulo_to_args,
                    )?;
                    segment_ptr
                }
                _ => {
                    let mut value: MaybeRelocatable = arg.extract::<PyMaybeRelocatable>(py)?.into();
                    if apply_modulo_to_args {
                        value = value
                            .mod_floor(self.pyvm.vm.borrow().get_prime())
                            .map_err(to_py_error)?;
                    }
                    value
                }
            })
            .to_object(py),
        )
    }

    #[args(apply_modulo_to_args = true)]
    pub fn write_arg(
        &self,
        py: Python<'_>,
        ptr: PyMaybeRelocatable,
        arg: Py<PyAny>,
        apply_modulo_to_args: bool,
    ) -> PyResult<PyObject> {
        let ptr: MaybeRelocatable = ptr.into();

        let arg_iter = PyIterator::from_object(py, &arg)?;
        let mut data = Vec::new();
        for value in arg_iter {
            data.push(
                self.gen_arg(py, value?.to_object(py), apply_modulo_to_args)?
                    .extract::<PyMaybeRelocatable>(py)?
                    .into(),
            );
        }

        (*self.pyvm.vm)
            .borrow_mut()
            .load_data(&ptr, data)
            .map(|x| PyMaybeRelocatable::from(x).to_object(py))
            .map_err(to_py_error)
    }

    /// Return a value from memory given its address.
    pub fn get(&self, py: Python, key: &PyRelocatable) -> PyResult<Option<PyObject>> {
        self.pyvm
            .vm
            .borrow()
            .get_maybe(key)
            .map_err(to_py_error)
            .map(|x| x.map(|x| PyMaybeRelocatable::from(x).to_object(py)))
    }

    /// Return a list of values from memory given an initial address and a length.
    pub fn get_range(&self, py: Python, key: &PyRelocatable, size: usize) -> PyResult<PyObject> {
        Ok(self
            .pyvm
            .vm
            .borrow()
            .get_continuous_range(&MaybeRelocatable::RelocatableValue(key.into()), size)
            .map_err(to_py_error)?
            .into_iter()
            .map(PyMaybeRelocatable::from)
            .collect::<Vec<_>>()
            .to_object(py))
    }

    /**  Converts typed arguments to cairo friendly ones
    The args received should be an iterable with an __annotations__ attribute with a values method
    which returns an iterable containing the types of each of the elements in args
    These types should de TypePointer, TypeFelt or TypeStruct
    This method is meant to process starknet's current typed arguments structure and shouldnt be used in any other case
    **/
    fn gen_typed_args(&self, py: Python<'_>, args: Py<PyAny>) -> PyResult<PyObject> {
        let args_iter = PyIterator::from_object(py, &args)?;
        let annotations_values = args
            .getattr(py, "__annotations__")?
            .call_method0(py, "values")?;

        let annotation_values = PyIterator::from_object(py, &annotations_values);

        let mut cairo_args = Vec::new();
        for (value, field_type) in std::iter::zip(args_iter, annotation_values?) {
            let type_str = format!("{:?}", field_type?);
            let type_str = type_str
                .rsplit('.')
                .next()
                .ok_or_else(|| PyTypeError::new_err("gen_typed_args: Failed to get arg type"))?
                .trim_end_matches("'>");

            if type_str == "TypePointer" || type_str == "TypeFelt" {
                cairo_args.push(self.gen_arg(py, value?.to_object(py), true)?)
            } else if type_str == "TypeStruct" {
                cairo_args.extend(self.gen_typed_args(py, value?.to_object(py)));
            } else {
                return Err(PyValueError::new_err(format!(
                    "Failed to generate typed arguments: {:?} is not supported",
                    type_str
                )));
            }
        }

        Ok(cairo_args.to_object(py))
    }

    /// Add (or replace if already present) a custom hash builtin.
    /// Returns a Relocatable with the new hash builtin base.
    pub fn add_additional_hash_builtin(&self) -> PyRelocatable {
        let mut vm = (*self.pyvm.vm).borrow_mut();
        self.inner.add_additional_hash_builtin(&mut vm).into()
    }

    #[getter]
    pub fn memory(&self) -> PyMemory {
        PyMemory::new(&self.pyvm)
    }

    #[getter]
    pub fn vm(&self) -> PyVM {
        self.pyvm.clone()
    }

    #[getter]
    pub fn vm_memory(&self) -> PyMemory {
        PyMemory::new(&self.pyvm)
    }
}

#[pyclass]
pub struct PyExecutionResources(ExecutionResources);

#[pymethods]
impl PyExecutionResources {
    #[getter]
    fn n_steps(&self) -> usize {
        self.0.n_steps
    }

    #[getter]
    fn n_memory_holes(&self) -> usize {
        self.0.n_memory_holes
    }

    #[getter]
    fn builtin_instance_counter(&self) -> HashMap<String, usize> {
        let mut instance_counters = self.0.builtin_instance_counter.clone();
        // replace the builtin name with "<name>_builtin" as expected in the Starknet code.
        for builtin_name in self.0.builtin_instance_counter.keys() {
            if let Some((key, counter)) = instance_counters.remove_entry(builtin_name) {
                instance_counters
                    .entry(format!("{}_builtin", key).to_string())
                    .or_insert(counter);
            }
        }

        instance_counters
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cairo_rs_py::relocatable::PyMaybeRelocatable::RelocatableValue;
    use cairo_rs::bigint;
    use num_bigint::BigInt;
    use pyo3::PyIterProtocol;
    use std::fs;

    #[test]
    fn create_cairo_runner() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
    }

    #[test]
    fn initialize_runner() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
        runner.initialize().unwrap();
    }

    #[test]
    fn runner_relocate() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
        runner.relocate().unwrap();
    }

    #[test]
    fn get_output() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();
        runner.get_output().unwrap();
    }

    #[test]
    fn write_output() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();
        runner.write_output().unwrap();
    }

    #[test]
    fn get_ap() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();
        assert_eq!(runner.get_ap().unwrap(), PyRelocatable::from((1, 0)));
    }

    #[test]
    fn add_segment() {
        let path = "cairo_programs/get_builtins_initial_stack.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();
        let new_segment = runner.add_segment();
        assert_eq!(
            new_segment,
            PyRelocatable {
                segment_index: 5,
                offset: 0
            }
        );
        let new_segment = runner.add_segment();
        assert_eq!(
            new_segment,
            PyRelocatable {
                segment_index: 6,
                offset: 0
            }
        );
    }

    #[test]
    fn get_builtins_initial_stack() {
        let path = "cairo_programs/get_builtins_initial_stack.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

        let expected_output: Vec<PyMaybeRelocatable> = vec![RelocatableValue(PyRelocatable {
            segment_index: 2,
            offset: 0,
        })];

        Python::with_gil(|py| {
            assert_eq!(
                runner
                    .get_program_builtins_initial_stack(py)
                    .extract::<Vec<PyMaybeRelocatable>>(py)
                    .unwrap(),
                expected_output
            );
        });
    }

    #[test]
    fn get_builtins_initial_stack_two_builtins() {
        let path = "cairo_programs/keccak_copy_inputs.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

        let expected_output: Vec<PyMaybeRelocatable> = vec![
            RelocatableValue(PyRelocatable {
                segment_index: 2,
                offset: 0,
            }),
            RelocatableValue(PyRelocatable {
                segment_index: 3,
                offset: 0,
            }),
        ];

        Python::with_gil(|py| {
            assert_eq!(
                runner
                    .get_program_builtins_initial_stack(py)
                    .extract::<Vec<PyMaybeRelocatable>>(py)
                    .unwrap(),
                expected_output
            );
        });
    }

    #[test]
    fn get_builtins_final_stack() {
        let path = "cairo_programs/get_builtins_initial_stack.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

        let expected_output = PyRelocatable::from((1, 8));

        let final_stack = PyRelocatable::from((1, 9));
        assert_eq!(
            runner.get_builtins_final_stack(final_stack).unwrap(),
            expected_output
        );
    }

    #[test]
    fn get_builtins_initial_stack_filters_non_program_builtins() {
        let path = "cairo_programs/get_builtins_initial_stack.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();
        runner
            .cairo_run_py(false, None, None, None, None, Some("main"))
            .unwrap();
        // Make a copy of the builtin in order to insert a second "fake" one
        // BuiltinRunner api is private, so we can create a new one for this test
        let fake_builtin = (*runner.pyvm.vm).borrow_mut().get_builtin_runners_as_mut()[0]
            .1
            .clone();
        // Insert our fake builtin into our vm
        (*runner.pyvm.vm)
            .borrow_mut()
            .get_builtin_runners_as_mut()
            .push((String::from("fake"), fake_builtin));
        // The fake builtin we added should be filtered out when getting the initial stacks,
        // so we should only get the range_check builtin's initial stack
        let expected_output: Vec<PyMaybeRelocatable> = vec![RelocatableValue(PyRelocatable {
            segment_index: 2,
            offset: 0,
        })];

        Python::with_gil(|py| {
            assert_eq!(
                runner
                    .get_program_builtins_initial_stack(py)
                    .extract::<Vec<PyMaybeRelocatable>>(py)
                    .unwrap(),
                expected_output
            );
        });
    }

    #[test]
    fn final_stack_when_not_using_builtins() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

        let expected_output = PyRelocatable::from((1, 0));

        let final_stack = PyRelocatable::from((1, 0));
        assert_eq!(
            runner.get_builtins_final_stack(final_stack).unwrap(),
            expected_output
        );
    }
    #[test]
    fn get_builtins_final_stack_filters_non_program_builtins() {
        let path = "cairo_programs/get_builtins_initial_stack.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("small".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

        // Make a copy of the builtin in order to insert a second "fake" one
        // BuiltinRunner api is private, so we can create a new one for this test
        let fake_builtin = (*runner.pyvm.vm).borrow_mut().get_builtin_runners_as_mut()[0]
            .1
            .clone();
        // Insert our fake builtin into our vm
        (*runner.pyvm.vm)
            .borrow_mut()
            .get_builtin_runners_as_mut()
            .push((String::from("fake"), fake_builtin));
        // The fake builtin we added should be filtered out when getting the final stacks,
        // so we should only get the range_check builtin's final stack

        let expected_output = PyRelocatable::from((1, 8));

        let final_stack = PyRelocatable::from((1, 9));
        assert_eq!(
            runner.get_builtins_final_stack(final_stack).unwrap(),
            expected_output
        );
    }

    #[test]
    fn final_stack_when_using_two_builtins() {
        let path = "cairo_programs/final_stack.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all".to_string()),
            false,
        )
        .unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();

        assert_eq!(runner.pyvm.vm.borrow().get_ap(), Relocatable::from((1, 41)));
        assert_eq!(
            runner
                .pyvm
                .vm
                .borrow()
                .get_maybe(&Relocatable::from((1, 40)))
                .unwrap()
                .unwrap(),
            MaybeRelocatable::from((3, 20))
        );
        assert_eq!(
            runner
                .pyvm
                .vm
                .borrow()
                .get_maybe(&Relocatable::from((1, 39)))
                .unwrap()
                .unwrap(),
            MaybeRelocatable::from((2, 0))
        );

        let expected_output = PyRelocatable::from((1, 39));
        let final_stack = PyRelocatable::from((1, 41));

        assert_eq!(
            runner.get_builtins_final_stack(final_stack).unwrap(),
            expected_output
        );
    }

    #[test]
    fn get_segment_used_size_of_segment_0() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();
        Python::with_gil(|py| {
            assert_eq!(
                24,
                runner
                    .get_segment_used_size(0, py)
                    .unwrap()
                    .extract::<usize>(py)
                    .unwrap()
            )
        });
    }

    #[test]
    fn get_segment_used_size_of_segment_2() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();
        Python::with_gil(|py| {
            assert_eq!(
                0,
                runner
                    .get_segment_used_size(2, py)
                    .unwrap()
                    .extract::<usize>(py)
                    .unwrap()
            )
        });
    }

    #[test]
    fn run_from_entrypoint_without_args() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        // Without `runner.initialize()`, an uninitialized error is returned.
        // With `runner.initialize()`, an invalid memory assignment is returned...
        //   Maybe it has to do with `initialize_main_entrypoint()` called from `initialize()`?
        runner.initialize_segments();

        Python::with_gil(|py| {
            runner
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
        });
    }

    #[test]
    fn run_from_entrypoint_without_args_set_hint_locals() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner.initialize_segments();

        Python::with_gil(|py| {
            runner
                .run_from_entrypoint(
                    py,
                    py.eval("0", None, None).unwrap(),
                    Vec::<&PyAny>::new().to_object(py),
                    Some(HashMap::from([(
                        String::from("syscall_handler"),
                        1.to_object(py),
                    )])),
                    None,
                    Some(false),
                    None,
                    None,
                )
                .unwrap();
            assert!(!runner.hint_locals.is_empty());
            assert_eq!(
                runner
                    .hint_locals
                    .get("syscall_handler")
                    .unwrap()
                    .extract::<usize>(py)
                    .unwrap(),
                1
            )
        });
    }

    #[test]
    fn run_from_entrypoint_without_args_set_static_locals() {
        let path = "cairo_programs/not_main.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("plain".to_string()),
            false,
        )
        .unwrap();

        runner.initialize_segments();

        Python::with_gil(|py| {
            runner
                .run_from_entrypoint(
                    py,
                    py.eval("0", None, None).unwrap(),
                    Vec::<&PyAny>::new().to_object(py),
                    None,
                    Some(HashMap::from([(
                        String::from("__keccak_max_size"),
                        100.to_object(py),
                    )])),
                    Some(false),
                    None,
                    None,
                )
                .unwrap();
            assert!(!runner.static_locals.as_ref().unwrap().is_empty());
            assert_eq!(
                runner
                    .static_locals
                    .as_ref()
                    .unwrap()
                    .get("__keccak_max_size")
                    .unwrap()
                    .extract::<usize>(py)
                    .unwrap(),
                100
            )
        });
    }

    #[test]
    fn run_from_entrypoint_with_one_typed_arg() {
        // One arg (typed)
        //   value
    }

    #[test]
    fn run_from_entrypoint_with_one_typed_vec_arg() {
        // One arg (typed)
        //   vec
    }

    #[test]
    fn run_from_entrypoint_with_multiple_untyped_args() {
        // Multiple args (no typed)
        // Test that `PyCairoRunner::insert()` inserts values correctly.
    }

    #[test]
    fn insert() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let runner = PyCairoRunner::new(program, Some("main".to_string()), None, true).unwrap();

        (*runner.pyvm.get_vm()).borrow_mut().add_memory_segment();
        runner
            .insert(&(0, 0).into(), PyMaybeRelocatable::Int(bigint!(3)))
            .unwrap();
        runner
            .insert(&(0, 1).into(), PyMaybeRelocatable::Int(bigint!(4)))
            .unwrap();
        runner
            .insert(&(0, 2).into(), PyMaybeRelocatable::Int(bigint!(5)))
            .unwrap();
        assert_eq!(
            (*runner.pyvm.get_vm())
                .borrow()
                .get_continuous_range(&(0, 0).into(), 3),
            Ok(vec![
                bigint!(3).into(),
                bigint!(4).into(),
                bigint!(5).into(),
            ]),
        )
    }

    /// Test that `PyCairoRunner::insert()` fails when it should.
    #[test]
    fn insert_duplicate() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let runner = PyCairoRunner::new(program, Some("main".to_string()), None, true).unwrap();

        (*runner.pyvm.get_vm()).borrow_mut().add_memory_segment();
        runner
            .insert(&(0, 0).into(), PyMaybeRelocatable::Int(bigint!(3)))
            .unwrap();
        runner
            .insert(&(0, 1).into(), PyMaybeRelocatable::Int(bigint!(4)))
            .unwrap();
        runner
            .insert(&(0, 0).into(), PyMaybeRelocatable::Int(bigint!(5)))
            .expect_err("insertion succeeded when it should've failed");
        assert_eq!(
            (*runner.pyvm.get_vm())
                .borrow()
                .get_continuous_range(&(0, 0).into(), 2),
            Ok(vec![bigint!(3).into(), bigint!(4).into(),]),
        );
    }

    #[test]
    fn get_initial_fp_test() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some(String::from("all")),
            false,
        )
        .unwrap();
        runner
            .cairo_run_py(false, None, None, None, None, None)
            .unwrap();
        assert_eq! {
            PyRelocatable::from((1,2)),
            runner.initial_fp().unwrap()
        };
    }

    #[test]
    fn initialize_function_runner() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all".to_string()),
            false,
        )
        .unwrap();

        runner.initialize_function_runner().unwrap();

        let expected_output: Vec<Vec<PyMaybeRelocatable>> = vec![
            vec![RelocatableValue(PyRelocatable {
                segment_index: 2,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 3,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 4,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 5,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 6,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 7,
                offset: 0,
            })],
            vec![RelocatableValue(PyRelocatable {
                segment_index: 8,
                offset: 0,
            })],
        ];

        Python::with_gil(|py| {
            assert_eq!(
                runner
                    .get_builtins_initial_stack(py)
                    .extract::<Vec<Vec<PyMaybeRelocatable>>>(py)
                    .unwrap(),
                expected_output
            );
        });
    }

    #[test]
    fn program_builtins_initial_stack_are_empty_when_no_program_builtins() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all".to_string()),
            false,
        )
        .unwrap();

        runner.initialize_function_runner().unwrap();

        let expected_output: Vec<Vec<PyMaybeRelocatable>> = vec![];

        Python::with_gil(|py| {
            assert_eq!(
                runner
                    .get_program_builtins_initial_stack(py)
                    .extract::<Vec<Vec<PyMaybeRelocatable>>>(py)
                    .unwrap(),
                expected_output
            );
        });
    }

    #[test]
    fn write_arg_test() {
        Python::with_gil(|py| {
            let path = "cairo_programs/fibonacci.json".to_string();
            let program = fs::read_to_string(path).unwrap();
            let runner = PyCairoRunner::new(
                program,
                Some("main".to_string()),
                Some("all".to_string()),
                false,
            )
            .unwrap();

            let ptr = runner.add_segment();
            runner
                .write_arg(
                    py,
                    PyMaybeRelocatable::RelocatableValue(ptr),
                    py.eval("[1, 2, [3, 4], [5, 6]]", None, None)
                        .unwrap()
                        .to_object(py),
                    true,
                )
                .unwrap();

            let vm_ref = runner.pyvm.get_vm();
            let vm_ref = (*vm_ref).borrow();

            assert_eq!(
                vm_ref
                    .get_maybe(&Relocatable::from((0, 0)))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(1),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&Relocatable::from((0, 1)))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(2),
            );

            let relocatable = vm_ref
                .get_maybe(&Relocatable::from((0, 2)))
                .unwrap()
                .unwrap()
                .get_relocatable()
                .unwrap()
                .clone();

            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable)
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(3),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&(&relocatable + 1))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(4),
            );
            assert!(vm_ref.get_maybe(&(&relocatable + 2)).unwrap().is_none());

            let relocatable = vm_ref
                .get_maybe(&Relocatable::from((0, 3)))
                .unwrap()
                .unwrap()
                .get_relocatable()
                .unwrap()
                .clone();

            assert_eq!(
                vm_ref
                    .get_maybe(&relocatable)
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(5),
            );
            assert_eq!(
                vm_ref
                    .get_maybe(&(&relocatable + 1))
                    .unwrap()
                    .unwrap()
                    .get_int_ref()
                    .unwrap(),
                &bigint!(6),
            );
            assert!(vm_ref.get_maybe(&(&relocatable + 2)).unwrap().is_none());

            assert!(vm_ref
                .get_maybe(&Relocatable::from((0, 4)))
                .unwrap()
                .is_none());
        });
    }

    #[test]
    fn run_find_element_with_max_size() {
        let path = "cairo_programs/find_element.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all".to_string()),
            false,
        )
        .unwrap();
        assert!(runner
            .cairo_run_py(
                false,
                None,
                None,
                None,
                Some(HashMap::from([(
                    "__find_element_max_size".to_string(),
                    Python::with_gil(|py| -> PyObject { 100.to_object(py) }),
                )])),
                None,
            )
            .is_ok());
    }

    #[test]
    fn run_find_element_with_max_size_low_size() {
        let path = "cairo_programs/find_element.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some("all".to_string()),
            false,
        )
        .unwrap();
        assert!(runner
            .cairo_run_py(
                false,
                None,
                None,
                None,
                Some(HashMap::from([(
                    "__find_element_max_size".to_string(),
                    Python::with_gil(|py| -> PyObject { 1.to_object(py) }),
                )])),
                None
            )
            .is_err());
    }

    #[test]
    fn set_entrypoint() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, None, Some("small".to_string()), false).unwrap();

        runner
            .cairo_run_py(false, None, None, None, None, Some("main"))
            .expect("Call to PyCairoRunner::cairo_run_py() failed.");
    }

    /// Test that `PyCairoRunner::get()` works as intended.
    #[test]
    fn get() {
        Python::with_gil(|py| {
            let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
            let mut runner = PyCairoRunner::new(
                program,
                Some("main".to_string()),
                Some("small".to_string()),
                false,
            )
            .unwrap();

            runner
                .cairo_run_py(false, None, None, None, None, None)
                .expect("Call to PyCairoRunner::cairo_run_py");

            let mut ap = runner.get_ap().unwrap();
            ap.offset -= 1;
            assert_eq!(
                runner
                    .get(py, &ap)
                    .unwrap()
                    .map(|x| MaybeRelocatable::from(x.extract::<PyMaybeRelocatable>(py).unwrap())),
                Some(MaybeRelocatable::Int(bigint!(144))),
            );
        });
    }

    /// Test that `PyCairoRunner::get_range()` works as intended.
    #[test]
    fn get_range() {
        Python::with_gil(|py| {
            let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
            let runner = PyCairoRunner::new(
                program,
                Some("main".to_string()),
                Some("small".to_string()),
                false,
            )
            .unwrap();

            let ptr = {
                let mut vm = (*runner.pyvm.vm).borrow_mut();
                let ptr = vm.add_memory_segment();
                vm.load_data(
                    &(&ptr).into(),
                    vec![
                        bigint!(1).into(),
                        bigint!(2).into(),
                        bigint!(3).into(),
                        bigint!(4).into(),
                        bigint!(5).into(),
                    ],
                )
                .unwrap();

                ptr
            };

            assert_eq!(
                runner
                    .get_range(py, &ptr.into(), 5)
                    .unwrap()
                    .extract::<Vec<PyMaybeRelocatable>>(py)
                    .unwrap()
                    .into_iter()
                    .map(MaybeRelocatable::from)
                    .collect::<Vec<_>>(),
                vec![
                    bigint!(1).into(),
                    bigint!(2).into(),
                    bigint!(3).into(),
                    bigint!(4).into(),
                    bigint!(5).into(),
                ],
            );
        });
    }

    /// Test that add_additional_hash_builtin() returns successfully.
    #[test]
    fn add_additional_hash_builtin() {
        Python::with_gil(|_| {
            let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
            let runner = PyCairoRunner::new(
                program,
                Some("main".to_string()),
                Some("small".to_string()),
                false,
            )
            .unwrap();

            let expected_relocatable = PyRelocatable {
                segment_index: 0,
                offset: 0,
            };
            let relocatable = runner.add_additional_hash_builtin();
            assert_eq!(expected_relocatable, relocatable);

            assert_eq!(
                (*runner.pyvm.vm)
                    .borrow()
                    .get_builtin_runners()
                    .last()
                    .map(|(key, _)| key.as_str()),
                Some("hash_builtin"),
            );

            let mut vm = (*runner.pyvm.vm).borrow_mut();
            // Check that the segment exists by writing to it.
            vm.insert_value(
                &Relocatable::from((0, 0)),
                MaybeRelocatable::Int(bigint!(42)),
            )
            .expect("memory insert failed");
        });
    }

    #[test]
    fn gen_typed_args_type_felt() {
        /* First we need to create a structure that behaves similarly to starknet's typed args
        This means we need:
        A: An iterable object
        B: An object that has an __annotations__ attribute
        C: The __annotations__  attribute should have a values method
        D: Values must return an iterable object containing the arg's type for each of the elements in args
        F: This iterable object must yield the following format string when format!("{:?") is applied to it:
            **.Type or **.Type'>
        Where Type can be either TypeFelt, TypePointer or TypeStruct
        */

        // We first create the iterable pyclass (A), implementing PyIterProtocol
        #[pyclass(unsendable)]
        struct MyIterator {
            iter: Box<dyn Iterator<Item = PyObject>>,
        }

        #[pyproto]
        impl PyIterProtocol for MyIterator {
            fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
                slf
            }
            fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
                slf.iter.next()
            }
        }

        // We then implement a __getattr__ that will allow us to call Object.__annotations__ (B)
        // This method returns a second object, so that we can then implement the values() method

        #[pymethods]
        // This method is implemented exclusively to support arg.__annotations__
        impl MyIterator {
            fn __getattr__(&self, _name: String) -> PyResult<Annotations> {
                Ok(Annotations {
                    0: vec![TypeFelt, TypeFelt],
                })
            }
        }
        #[pyclass(unsendable)]
        struct Annotations(Vec<TypeFelt>);

        // We implement the values method (C), which in turn returns another object so that we can override its representation
        #[pymethods]
        impl Annotations {
            pub fn values(&self) -> PyResult<Vec<TypeFelt>> {
                Ok(self.0.clone())
            }
        }

        #[pyclass]
        #[derive(Clone)]
        struct TypeFelt;

        // We override the __repr__ method, so that we can customize the string we get when calling format!({:?}) (F)
        #[pymethods]
        impl TypeFelt {
            fn __repr__(&self) -> String {
                format!("TypeFelt")
            }
        }

        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();
        Python::with_gil(|py| {
            // We create an iterable object containing elements which match the type we defined in (F), thus fullfilling (D)
            let arg = MyIterator {
                iter: Box::new(
                    vec![
                        PyMaybeRelocatable::from(bigint!(0)).to_object(py),
                        PyMaybeRelocatable::from(bigint!(2)).to_object(py),
                    ]
                    .into_iter(),
                ),
            };
            let stack = runner.gen_typed_args(py, arg.into_py(py)).unwrap();
            let stack = stack.extract::<Vec<PyMaybeRelocatable>>(py).unwrap();

            // We compare the output of gen_typed_args to our expected cairo-firendly arguments
            assert_eq!(
                stack,
                vec![
                    PyMaybeRelocatable::from(bigint!(0)),
                    PyMaybeRelocatable::from(bigint!(2)),
                ]
            );
        })
    }

    #[test]
    fn gen_typed_args_type_pointer() {
        //For documentation on how this test works see gen_typed_args_type_pointer()
        #[pyclass(unsendable)]
        struct MyIterator {
            iter: Box<dyn Iterator<Item = PyObject>>,
        }

        #[pyproto]
        impl PyIterProtocol for MyIterator {
            fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
                slf
            }
            fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
                slf.iter.next()
            }
        }
        #[pymethods]
        // This method is implemented exclusively to support arg.__annotations__
        impl MyIterator {
            fn __getattr__(&self, _name: String) -> PyResult<Annotations> {
                Ok(Annotations {
                    0: vec![TypePointer, TypePointer],
                })
            }
        }
        #[pyclass(unsendable)]
        struct Annotations(Vec<TypePointer>);

        #[pymethods]
        impl Annotations {
            pub fn values(&self) -> PyResult<Vec<TypePointer>> {
                Ok(self.0.clone())
            }
        }

        #[pyclass]
        #[derive(Clone)]
        struct TypePointer;
        #[pymethods]
        impl TypePointer {
            fn __repr__(&self) -> String {
                format!("TypePointer")
            }
        }

        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();
        Python::with_gil(|py| {
            let arg = MyIterator {
                iter: Box::new(
                    vec![
                        Into::<PyMaybeRelocatable>::into(MaybeRelocatable::from((0, 0)))
                            .to_object(py),
                        Into::<PyMaybeRelocatable>::into(MaybeRelocatable::from((0, 1)))
                            .to_object(py),
                    ]
                    .into_iter(),
                ),
            };
            let stack = runner.gen_typed_args(py, arg.into_py(py)).unwrap();
            let stack = stack.extract::<Vec<PyMaybeRelocatable>>(py).unwrap();
            assert_eq!(
                stack,
                vec![
                    MaybeRelocatable::from((0, 0)).into(),
                    MaybeRelocatable::from((0, 1)).into(),
                ]
            );
        })
    }

    #[test]
    fn memory() {
        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();

        let memory = runner.memory();

        Python::with_gil(|py| {
            let segment = runner.add_segment();

            let set_value = |addr: &PyRelocatable, offset, value: BigInt| {
                let addr = addr.__add__(offset);
                memory
                    .__setitem__(&addr, PyMaybeRelocatable::Int(value))
                    .expect("Could not insert value into memory.");
            };
            let get_value = |addr: &PyRelocatable, offset| {
                let addr = addr.__add__(offset);
                memory
                    .__getitem__(&addr, py)
                    .expect("Could not get value from memory.")
                    .map(|x| x.extract::<BigInt>(py))
                    .transpose()
                    .expect("Could not convert value to a BigInt")
            };

            set_value(&segment, 0, bigint!(1));
            set_value(&segment, 1, bigint!(2));
            set_value(&segment, 2, bigint!(3));
            set_value(&segment, 3, bigint!(4));

            assert_eq!(get_value(&segment, 0), Some(bigint!(1)));
            assert_eq!(get_value(&segment, 1), Some(bigint!(2)));
            assert_eq!(get_value(&segment, 2), Some(bigint!(3)));
            assert_eq!(get_value(&segment, 3), Some(bigint!(4)));
            assert_eq!(get_value(&segment, 4), None);
        });
    }

    #[test]
    fn vm() {
        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();

        let vm = runner.vm();
        assert_eq!(vm.vm.as_ptr(), runner.pyvm.vm.as_ptr());
    }

    #[test]
    fn vm_memory() {
        let program = fs::read_to_string("cairo_programs/fibonacci.json").unwrap();
        let runner = PyCairoRunner::new(program, None, None, false).unwrap();

        let memory = runner.vm_memory();

        Python::with_gil(|py| {
            let segment = runner.add_segment();

            let set_value = |addr: &PyRelocatable, offset, value: BigInt| {
                let addr = addr.__add__(offset);
                memory
                    .__setitem__(&addr, PyMaybeRelocatable::Int(value))
                    .expect("Could not insert value into memory.");
            };
            let get_value = |addr: &PyRelocatable, offset| {
                let addr = addr.__add__(offset);
                memory
                    .__getitem__(&addr, py)
                    .expect("Could not get value from memory.")
                    .map(|x| x.extract::<BigInt>(py))
                    .transpose()
                    .expect("Could not convert value to a BigInt")
            };

            set_value(&segment, 0, bigint!(1));
            set_value(&segment, 1, bigint!(2));
            set_value(&segment, 2, bigint!(3));
            set_value(&segment, 3, bigint!(4));

            assert_eq!(get_value(&segment, 0), Some(bigint!(1)));
            assert_eq!(get_value(&segment, 1), Some(bigint!(2)));
            assert_eq!(get_value(&segment, 2), Some(bigint!(3)));
            assert_eq!(get_value(&segment, 3), Some(bigint!(4)));
            assert_eq!(get_value(&segment, 4), None);
        });
    }
}
