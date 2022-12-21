use crate::cairo_rs_py::ecdsa::PySignature;
use crate::cairo_rs_py::ids::PyIds;
use crate::pycell;
use crate::cairo_rs_py::run_context::PyRunContext;
use crate::cairo_rs_py::scope_manager::{PyEnterScope, PyExitScope};
use crate::cairo_rs_py::to_felt_or_relocatable::ToFeltOrRelocatableFunc;
use crate::{
    cairo_rs_py::memory::PyMemory, cairo_rs_py::memory_segments::PySegmentManager, cairo_rs_py::range_check::PyRangeCheck,
    cairo_rs_py::relocatable::PyRelocatable, cairo_rs_py::utils::to_vm_error,
};
use cairo_rs::any_box;
use cairo_rs::hint_processor::hint_processor_definition::HintProcessor;
use cairo_rs::serde::deserialize_program::{Attribute, Member};
use cairo_rs::types::exec_scope::ExecutionScopes;
use cairo_rs::vm::vm_core::VirtualMachine;
use cairo_rs::{
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData,
    vm::errors::vm_errors::VirtualMachineError,
};
use num_bigint::BigInt;
use pyo3::PyCell;
use pyo3::{pyclass, pymethods, PyObject, ToPyObject};
use pyo3::{types::PyDict, Python};
use std::any::Any;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

const GLOBAL_NAMES: [&str; 18] = [
    "memory",
    "segments",
    "ap",
    "fp",
    "ids",
    "vm_enter_scope",
    "vm_exit_scope",
    "to_felt_or_relocatable",
    "range_check_builtin",
    "ecdsa_builtin",
    "PRIME",
    "__doc__",
    "__annotations__",
    "__package__",
    "__builtins__",
    "__spec__",
    "__loader__",
    "__name__",
];

#[derive(Clone)]
#[pyclass(unsendable)]
pub struct PyVM {
    pub(crate) vm: Rc<RefCell<VirtualMachine>>,
}

#[pymethods]
impl PyVM {
    #[getter]
    fn run_context(&self) -> PyRunContext {
        let vm = self.vm.borrow();
        PyRunContext::new(vm.get_pc().clone(), vm.get_ap(), vm.get_fp())
    }
}

impl PyVM {
    pub fn new(
        prime: BigInt,
        trace_enabled: bool,
        error_message_attributes: Vec<Attribute>,
    ) -> PyVM {
        PyVM {
            vm: Rc::new(RefCell::new(VirtualMachine::new(
                prime,
                trace_enabled,
                error_message_attributes,
            ))),
        }
    }

    pub(crate) fn get_vm(&self) -> Rc<RefCell<VirtualMachine>> {
        Rc::clone(&self.vm)
    }

    pub(crate) fn execute_hint(
        &self,
        hint_data: &HintProcessorData,
        hint_locals: &mut HashMap<String, PyObject>,
        exec_scopes: &mut ExecutionScopes,
        constants: &HashMap<String, BigInt>,
        struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
        static_locals: Option<&HashMap<String, PyObject>>,
    ) -> Result<(), VirtualMachineError> {
        Python::with_gil(|py| -> Result<(), VirtualMachineError> {
            let memory = PyMemory::new(self);
            let segments = PySegmentManager::new(self, memory.clone());
            let ap = PyRelocatable::from((*self.vm).borrow().get_ap());
            let fp = PyRelocatable::from((*self.vm).borrow().get_fp());
            let ids = PyIds::new(
                self,
                &hint_data.ids_data,
                &hint_data.ap_tracking,
                constants,
                struct_types,
            );
            let enter_scope = pycell!(py, PyEnterScope::new());
            let exit_scope = pycell!(py, PyExitScope::new());
            let range_check_builtin =
                PyRangeCheck::from((*self.vm).borrow().get_range_check_builtin());
            let ecdsa_builtin = pycell!(py, PySignature::new());
            let prime = (*self.vm).borrow().get_prime().clone();
            let to_felt_or_relocatable = ToFeltOrRelocatableFunc;

            // This line imports Python builtins. If not imported, this will run only with Python 3.10
            let globals = py
                .import("__main__")
                .map_err(|err| to_vm_error(err, py))?
                .dict()
                .copy()
                .map_err(|err| to_vm_error(err, py))?;

            add_scope_locals(globals, exec_scopes, py)?;

            globals
                .set_item("memory", pycell!(py, memory))
                .map_err(|err| to_vm_error(err, py))?;
            globals
                .set_item("segments", pycell!(py, segments))
                .map_err(|err| to_vm_error(err, py))?;
            globals
                .set_item("ap", pycell!(py, ap))
                .map_err(|err| to_vm_error(err, py))?;
            globals
                .set_item("fp", pycell!(py, fp))
                .map_err(|err| to_vm_error(err, py))?;
            globals
                .set_item("ids", pycell!(py, ids))
                .map_err(|err| to_vm_error(err, py))?;
            globals
                .set_item("vm_enter_scope", enter_scope)
                .map_err(|err| to_vm_error(err, py))?;
            globals
                .set_item("vm_exit_scope", exit_scope)
                .map_err(|err| to_vm_error(err, py))?;
            globals
                .set_item("range_check_builtin", range_check_builtin)
                .map_err(|err| to_vm_error(err, py))?;
            globals
                .set_item("ecdsa_builtin", ecdsa_builtin)
                .map_err(|err| to_vm_error(err, py))?;
            globals
                .set_item("PRIME", prime)
                .map_err(|err| to_vm_error(err, py))?;
            globals
                .set_item(
                    "to_felt_or_relocatable",
                    pycell!(py, to_felt_or_relocatable),
                )
                .map_err(|err| to_vm_error(err, py))?;

            for (name, pyobj) in hint_locals.iter() {
                globals
                    .set_item(name, pyobj)
                    .map_err(|err| to_vm_error(err, py))?;
            }

            if let Some(static_locals) = static_locals {
                for (name, pyobj) in static_locals.iter() {
                    globals
                        .set_item(name, pyobj)
                        .map_err(|err| to_vm_error(err, py))?;
                }
            }

            py.run(&hint_data.code, Some(globals), None)
                .map_err(|err| to_vm_error(err, py))?;

            update_scope_hint_locals(exec_scopes, hint_locals, static_locals, globals, py);

            if self.vm.borrow_mut().get_signature_builtin().is_ok() {
                ecdsa_builtin
                    .borrow()
                    .update_signature(self.vm.borrow_mut().get_signature_builtin()?)?;
            }
            enter_scope.borrow().update_scopes(exec_scopes)?;
            exit_scope.borrow().update_scopes(exec_scopes)
        })?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step_hint(
        &self,
        hint_executor: &dyn HintProcessor,
        hint_locals: &mut HashMap<String, PyObject>,
        exec_scopes: &mut ExecutionScopes,
        hint_data_dictionary: &HashMap<usize, Vec<Box<dyn Any>>>,
        struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
        constants: &HashMap<String, BigInt>,
        static_locals: Option<&HashMap<String, PyObject>>,
    ) -> Result<(), VirtualMachineError> {
        let pc_offset = (*self.vm).borrow().get_pc().offset;

        if let Some(hint_list) = hint_data_dictionary.get(&pc_offset) {
            for hint_data in hint_list.iter() {
                if self.should_run_py_hint(hint_executor, exec_scopes, hint_data, constants)? {
                    let hint_data = hint_data
                        .downcast_ref::<HintProcessorData>()
                        .ok_or(VirtualMachineError::WrongHintData)?;

                    self.execute_hint(
                        hint_data,
                        hint_locals,
                        exec_scopes,
                        constants,
                        Rc::clone(&struct_types),
                        static_locals,
                    )?;
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step(
        &self,
        hint_executor: &dyn HintProcessor,
        hint_locals: &mut HashMap<String, PyObject>,
        exec_scopes: &mut ExecutionScopes,
        hint_data_dictionary: &HashMap<usize, Vec<Box<dyn Any>>>,
        struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
        constants: &HashMap<String, BigInt>,
        static_locals: Option<&HashMap<String, PyObject>>,
    ) -> Result<(), VirtualMachineError> {
        self.step_hint(
            hint_executor,
            hint_locals,
            exec_scopes,
            hint_data_dictionary,
            struct_types,
            constants,
            static_locals,
        )?;
        self.vm.borrow_mut().step_instruction()
    }

    fn should_run_py_hint(
        &self,
        hint_executor: &dyn HintProcessor,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, BigInt>,
    ) -> Result<bool, VirtualMachineError> {
        let mut vm = self.vm.borrow_mut();
        match hint_executor.execute_hint(&mut vm, exec_scopes, hint_data, constants) {
            Ok(()) => Ok(false),
            Err(VirtualMachineError::UnknownHint(_)) => Ok(true),
            Err(e) => Err(e),
        }
    }
}

pub(crate) fn add_scope_locals(
    globals: &PyDict,
    exec_scopes: &ExecutionScopes,
    py: Python,
) -> Result<(), VirtualMachineError> {
    for (name, elem) in exec_scopes.get_local_variables()? {
        if let Some(pyobj) = elem.downcast_ref::<PyObject>() {
            globals
                .set_item(name, pyobj)
                .map_err(|err| to_vm_error(err, py))?;
        }
    }
    Ok(())
}

pub(crate) fn update_scope_hint_locals(
    exec_scopes: &mut ExecutionScopes,
    hint_locals: &mut HashMap<String, PyObject>,
    static_locals: Option<&HashMap<String, PyObject>>,
    globals: &PyDict,
    py: Python,
) {
    let static_local_names = static_locals
        .map(|locals| locals.keys().collect::<Vec<&String>>())
        .unwrap_or_default();
    for (name, elem) in globals {
        let name = name.to_string();
        if !GLOBAL_NAMES.contains(&name.as_str()) && !static_local_names.contains(&&name) {
            if hint_locals.keys().cloned().any(|x| x == name) {
                hint_locals.insert(name, elem.to_object(py));
            } else {
                exec_scopes.assign_or_update_variable(&name, any_box!(elem.to_object(py)));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{relocatable::PyMaybeRelocatable, vm_core::PyVM};
    use cairo_rs::{
        bigint,
        hint_processor::{
            builtin_hint_processor::builtin_hint_processor_definition::{
                BuiltinHintProcessor, HintProcessorData,
            },
            hint_processor_definition::HintReference,
        },
        types::{
            exec_scope::ExecutionScopes,
            relocatable::{MaybeRelocatable, Relocatable},
        },
        vm::errors::{exec_scope_errors::ExecScopeError, vm_errors::VirtualMachineError},
    };
    use num_bigint::{BigInt, Sign};
    use pyo3::{PyObject, Python, ToPyObject};
    use std::{collections::HashMap, rc::Rc};

    #[test]
    fn execute_print_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let code = "print(ap)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
    }

    #[test]
    fn set_memory_item_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let code = "print(ap)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
    }

    #[test]
    fn ids_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }
        let references = HashMap::from([
            (String::from("a"), HintReference::new_simple(2)),
            (String::from("b"), HintReference::new_simple(1)),
        ]);
        vm.vm
            .borrow_mut()
            .insert_value(
                &Relocatable::from((1, 1)),
                &MaybeRelocatable::from(bigint!(2usize)),
            )
            .unwrap();
        let code = "ids.a = ids.b";
        let hint_data = HintProcessorData::new_default(code.to_string(), references);
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        assert_eq!(
            vm.vm.borrow().get_maybe(&Relocatable::from((1, 2))),
            Ok(Some(MaybeRelocatable::from(bigint!(2))))
        );
    }

    #[test]
    // Test the availability of cairo constants in ids
    fn const_ids() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );

        let constants = HashMap::from([(String::from("CONST"), bigint!(1))]);

        let mut exec_scopes = ExecutionScopes::new();
        let code_1 = "assert(ids.CONST != 2)";
        let hint_data = HintProcessorData::new_default(code_1.to_string(), HashMap::new());

        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &constants,
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );

        let code_2 = "assert(ids.CONST == 1)";
        let hint_data = HintProcessorData::new_default(code_2.to_string(), HashMap::new());

        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &constants,
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
    }

    #[test]
    // This test is analogous to the `test_step_for_preset_memory` unit test in the cairo-rs crate.
    fn test_step_with_no_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );

        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let hint_processor = BuiltinHintProcessor::new_empty();

        vm.vm.borrow_mut().set_pc(Relocatable::from((0, 0)));
        vm.vm.borrow_mut().set_ap(2);
        vm.vm.borrow_mut().set_fp(2);

        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((0, 0)), bigint!(2345108766317314046_u64))
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((1, 0)), &Relocatable::from((2, 0)))
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((1, 1)), &Relocatable::from((3, 0)))
            .unwrap();

        assert_eq!(
            vm.step(
                &hint_processor,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                &HashMap::new(),
                None,
            ),
            Ok(())
        );
    }

    #[test]
    fn test_step_with_print_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );

        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let hint_processor = BuiltinHintProcessor::new_empty();

        vm.vm.borrow_mut().set_pc(Relocatable::from((0, 0)));
        vm.vm.borrow_mut().set_ap(2);
        vm.vm.borrow_mut().set_fp(2);

        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((0, 0)), bigint!(2345108766317314046_u64))
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((1, 0)), &Relocatable::from((2, 0)))
            .unwrap();
        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((1, 1)), &Relocatable::from((3, 0)))
            .unwrap();

        let code = "print(ap)";
        let hint_proc_data = HintProcessorData::new_default(code.to_string(), HashMap::new());

        let mut hint_data = HashMap::new();
        hint_data.insert(0, hint_proc_data);

        assert_eq!(
            vm.step(
                &hint_processor,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                &HashMap::new(),
                None,
            ),
            Ok(())
        );
    }

    #[test]
    fn scopes_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let mut exec_scopes = ExecutionScopes::new();
        let code_a = "num = 6";
        let code_b = "assert(num == 6)";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());

        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
    }

    #[test]
    fn scopes_hint_modify() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        for _ in 0..2 {
            vm.vm.borrow_mut().add_memory_segment();
        }

        let mut exec_scopes = ExecutionScopes::new();
        let code_a = "num = 6";
        let code_b = "assert(num == 6)";
        let code_c = "num = num + 3";
        let code_d = "assert(num == 9)";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        let hint_data = HintProcessorData::new_default(code_c.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        let hint_data = HintProcessorData::new_default(code_d.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
    }

    #[test]
    fn modify_hint_locals() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let code = "word = word[::-1]
print(word)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        let word = Python::with_gil(|py| -> PyObject { "fruity".to_string().to_object(py) });
        let mut hint_locals = HashMap::from([("word".to_string(), word)]);
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut hint_locals,
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        let word_res = Python::with_gil(|py| -> String {
            hint_locals
                .get("word")
                .unwrap()
                .extract::<String>(py)
                .unwrap()
        });
        assert_eq!(word_res, "ytiurf".to_string())
    }

    #[test]
    fn exit_main_scope_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "vm_exit_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Err(VirtualMachineError::MainScopeError(
                ExecScopeError::ExitMainScopeError
            ))
        );
    }

    #[test]
    fn enter_scope_empty_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "vm_enter_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        assert_eq!(exec_scopes.data.len(), 2)
    }

    #[test]
    fn enter_exit_scope_same_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "vm_enter_scope()
vm_exit_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        assert_eq!(exec_scopes.data.len(), 1);
    }

    #[test]
    fn enter_exit_scope_separate_hints() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code_a = "vm_enter_scope()";
        let code_b = "vm_exit_scope()";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        assert_eq!(exec_scopes.data.len(), 2);
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        assert_eq!(exec_scopes.data.len(), 1)
    }

    #[test]
    fn enter_exit_enter_scope_same_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "vm_enter_scope()
vm_exit_scope()
vm_enter_scope()";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        assert_eq!(exec_scopes.data.len(), 2)
    }

    #[test]
    fn list_comprehension() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "lista_a = [1,2,3]
lista_b = [lista_a[k] for k in range(2)]";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
    }

    #[test]
    fn enter_scope_non_empty_hint() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code_a = "vm_enter_scope({'n': 12})";
        let code_b = "assert(n == 12)";
        let hint_data = HintProcessorData::new_default(code_a.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        let hint_data = HintProcessorData::new_default(code_b.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        assert_eq!(exec_scopes.data.len(), 2);
        assert!(exec_scopes.data[0].is_empty());
    }

    #[test]
    fn access_relocatable_segment_index() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "assert(ap.segment_index == 1)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
    }

    #[test]
    fn to_felt_or_relocatable_number() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "felt = to_felt_or_relocatable(456)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        Python::with_gil(|py| {
            assert_eq!(
                exec_scopes
                    .get_any_boxed_ref("felt")
                    .unwrap()
                    .downcast_ref::<PyObject>()
                    .unwrap()
                    .extract::<PyMaybeRelocatable>(py)
                    .unwrap(),
                PyMaybeRelocatable::from(bigint!(456))
            );
        });
    }

    #[test]
    fn to_felt_or_relocatable_list_should_fail() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "felt = to_felt_or_relocatable([1,2,3])";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        assert!(vm
            .execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            )
            .is_err());
    }

    #[test]
    fn to_felt_or_relocatable_relocatable() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "ids.test_value = to_felt_or_relocatable(ids.relocatable)";
        vm.vm.borrow_mut().add_memory_segment();
        vm.vm.borrow_mut().add_memory_segment();
        //insert ids.relocatable
        vm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((1, 0)), Relocatable::from((2, 0)))
            .unwrap();
        let ids = HashMap::from([
            ("relocatable".to_string(), HintReference::new_simple(0)),
            ("test_value".to_string(), HintReference::new_simple(1)),
        ]);
        let hint_data = HintProcessorData::new_default(code.to_string(), ids);
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        );
        //Check the value of ids.test_value
        assert_eq!(
            vm.vm
                .borrow()
                .get_relocatable(&Relocatable::from((1, 1)))
                .unwrap()
                .into_owned(),
            Relocatable::from((2, 0))
        );
    }

    #[test]
    fn test_get_range() {
        let pyvm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let mut exec_scopes = ExecutionScopes::new();
        let code = "assert(memory.get_range(ids.address, 3) == [1,2,7])";

        let ids = HashMap::from([("address".to_string(), HintReference::new_simple(0))]);

        for _ in 0..3 {
            pyvm.vm.borrow_mut().add_memory_segment();
        }

        pyvm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((1, 0)), Relocatable::from((2, 0)))
            .unwrap();

        pyvm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((2, 0)), bigint!(1))
            .unwrap();

        pyvm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((2, 1)), bigint!(2))
            .unwrap();

        pyvm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((2, 2)), bigint!(7))
            .unwrap();

        let hint_data = HintProcessorData::new_default(code.to_string(), ids);
        assert_eq!(
            pyvm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        )
    }

    #[test]
    fn test_segments_memory_get_range() {
        let pyvm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let code = "assert(segments.memory.get_range(ids.address, 2) == [9,12])";

        let ids = HashMap::from([("address".to_string(), HintReference::new_simple(0))]);

        for _ in 0..3 {
            pyvm.vm.borrow_mut().add_memory_segment();
        }

        pyvm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((1, 0)), Relocatable::from((2, 0)))
            .unwrap();

        pyvm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((2, 0)), bigint!(9))
            .unwrap();

        pyvm.vm
            .borrow_mut()
            .insert_value(&Relocatable::from((2, 1)), bigint!(12))
            .unwrap();

        let hint_data = HintProcessorData::new_default(code.to_string(), ids);
        assert_eq!(
            pyvm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut ExecutionScopes::new(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
                None,
            ),
            Ok(())
        )
    }

    #[test]
    fn run_hint_with_static_locals() {
        let mut vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let static_locals = HashMap::from([(
            "__number_max".to_string(),
            Python::with_gil(|py| -> PyObject { 90.to_object(py) }),
        )]);
        let code = "number = __number_max";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        let mut exec_scopes = ExecutionScopes::new();
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                Some(&static_locals),
            ),
            Ok(())
        );
        let number = Python::with_gil(|py| -> usize {
            exec_scopes.data[0]
                .get("number")
                .unwrap()
                .downcast_ref::<PyObject>()
                .unwrap()
                .extract::<usize>(py)
                .unwrap()
        });
        assert_eq!(number, 90)
    }

    #[test]
    fn run_hint_with_static_locals_shouldnt_change_its_value() {
        let mut vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let static_locals = HashMap::from([(
            "__number_max".to_string(),
            Python::with_gil(|py| -> PyObject { 90.to_object(py) }),
        )]);
        let code = "__number_max = 15";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        let mut exec_scopes = ExecutionScopes::new();
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut HashMap::new(),
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                Some(&static_locals),
            ),
            Ok(())
        );
        let number = Python::with_gil(|py| -> usize {
            static_locals
                .get("__number_max")
                .unwrap()
                .extract::<usize>(py)
                .unwrap()
        });
        assert_eq!(number, 90)
    }

    #[test]
    fn run_hint_with_static_locals_shouldnt_affect_scope_or_hint_locals() {
        let mut vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let static_locals = HashMap::from([(
            "__number_max".to_string(),
            Python::with_gil(|py| -> PyObject { 90.to_object(py) }),
        )]);
        let code = "assert(__number_max == 90)";
        let hint_data = HintProcessorData::new_default(code.to_string(), HashMap::new());
        let mut exec_scopes = ExecutionScopes::new();
        let mut hint_locals = HashMap::new();
        assert_eq!(
            vm.execute_hint(
                &hint_data,
                &mut hint_locals,
                &mut exec_scopes,
                &HashMap::new(),
                Rc::new(HashMap::new()),
                Some(&static_locals),
            ),
            Ok(())
        );
        assert!(exec_scopes.data[0].is_empty());
        assert!(hint_locals.is_empty())
    }

    #[test]
    fn run_context() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );

        let run_context = vm.run_context();
        assert_eq!(run_context.pc(), (0, 0).into());
        assert_eq!(run_context.ap(), (1, 0).into());
        assert_eq!(run_context.fp(), (1, 0).into());
    }
}
