use crate::{
    cairo_rs_py::relocatable::{PyMaybeRelocatable, PyRelocatable},
    cairo_rs_py::utils::to_py_error,
    cairo_rs_py::vm_core::PyVM,
};
use cairo_rs::{
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::vm_core::VirtualMachine,
};
use num_bigint::BigInt;
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
};
use std::{borrow::Cow, cell::RefCell, rc::Rc};

const MEMORY_GET_ERROR_MSG: &str = "Failed to get value from Cairo memory";
const MEMORY_SET_ERROR_MSG: &str = "Failed to set value to Cairo memory";
const MEMORY_GET_RANGE_ERROR_MSG: &str = "Failed to call get_range method from Cairo memory";
const MEMORY_ADD_RELOCATION_RULE_ERROR_MSG: &str =
    "Failed to call add_relocation_rule method from Cairo memory";

#[pyclass(unsendable)]
#[derive(Clone)]
pub struct PyMemory {
    vm: Rc<RefCell<VirtualMachine>>,
}

#[pymethods]
impl PyMemory {
    #[new]
    pub fn new(vm: &PyVM) -> PyMemory {
        PyMemory { vm: vm.get_vm() }
    }

    #[getter]
    pub fn __getitem__(&self, key: &PyRelocatable, py: Python) -> PyResult<Option<PyObject>> {
        match self
            .vm
            .borrow()
            .get_maybe(key)
            .map_err(|_| PyTypeError::new_err(MEMORY_GET_ERROR_MSG))?
        {
            Some(maybe_reloc) => Ok(Some(PyMaybeRelocatable::from(maybe_reloc).to_object(py))),
            None => Ok(None),
        }
    }

    #[setter]
    pub fn __setitem__(&self, key: &PyRelocatable, value: PyMaybeRelocatable) -> PyResult<()> {
        let key: Relocatable = key.into();
        let value: MaybeRelocatable = value.into();

        self.vm
            .borrow_mut()
            .insert_value(&key, value)
            .map_err(|_| PyValueError::new_err(MEMORY_SET_ERROR_MSG))
    }

    pub fn get_range(
        &self,
        addr: PyMaybeRelocatable,
        size: usize,
        py: Python,
    ) -> PyResult<PyObject> {
        Ok(self
            .vm
            .borrow()
            .get_continuous_range(&MaybeRelocatable::from(addr), size)
            .map_err(|_| PyTypeError::new_err(MEMORY_GET_RANGE_ERROR_MSG))?
            .into_iter()
            .map(Into::<PyMaybeRelocatable>::into)
            .collect::<Vec<PyMaybeRelocatable>>()
            .to_object(py))
    }

    pub fn add_relocation_rule(
        &self,
        src_ptr: PyRelocatable,
        dest_ptr: PyRelocatable,
    ) -> Result<(), PyErr> {
        self.vm
            .borrow_mut()
            .add_relocation_rule(Relocatable::from(&src_ptr), Relocatable::from(&dest_ptr))
            .map_err(|_| PyTypeError::new_err(MEMORY_ADD_RELOCATION_RULE_ERROR_MSG))
    }

    /// Return a continuous section of memory as a vector of integers.
    pub fn get_range_as_ints(&self, addr: PyRelocatable, size: usize) -> PyResult<Vec<BigInt>> {
        Ok(self
            .vm
            .borrow()
            .get_integer_range(&Relocatable::from(&addr), size)
            .map_err(to_py_error)?
            .into_iter()
            .map(Cow::into_owned)
            .collect())
    }
}

#[cfg(test)]
mod test {
    use crate::relocatable::PyMaybeRelocatable;
    use crate::relocatable::PyMaybeRelocatable::RelocatableValue;
    use crate::utils::to_vm_error;
    use crate::vm_core::PyVM;
    use crate::{memory::PyMemory, relocatable::PyRelocatable};
    use cairo_rs::bigint;
    use cairo_rs::types::relocatable::{MaybeRelocatable, Relocatable};
    use cairo_rs::vm::errors::vm_errors::VirtualMachineError;
    use num_bigint::{BigInt, Sign};
    use pyo3::PyCell;
    use pyo3::{types::PyDict, Python};

    #[test]
    fn memory_insert_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            let memory = PyMemory::new(&vm);
            let ap = PyRelocatable::from(vm.vm.borrow().get_ap());

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("ap", PyCell::new(py, ap).unwrap())
                .unwrap();

            let code = "memory[ap] = 5";

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(|err| to_vm_error(err, py)), Ok(()));
        });
    }

    #[test]
    fn memory_insert_ocuppied_address_error_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            let memory = PyMemory::new(&vm);
            let ap = PyRelocatable::from(vm.vm.borrow().get_ap());

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("ap", PyCell::new(py, ap).unwrap())
                .unwrap();

            // we try to insert to the same address two times
            let code = r#"
memory[ap] = 5
memory[ap] = 3
"#;

            let py_result = py.run(code, Some(globals), None);

            assert!(py_result.is_err());
        });
    }

    #[test]
    fn memory_get_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            let memory = PyMemory::new(&vm);
            let ap = PyRelocatable::from((1, 1));
            let fp = PyRelocatable::from((1, 2));

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("ap", PyCell::new(py, ap).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();

            let code = r#"
memory[ap] = fp
assert memory[ap] == fp
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(|err| to_vm_error(err, py)), Ok(()));
        });
    }

    #[test]
    fn get_range() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );

            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }

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

            let maybe_relocatable = MaybeRelocatable::from((1, 0));
            let size = 2;
            let memory = PyMemory::new(&vm);

            let range = memory
                .get_range(maybe_relocatable.into(), size, py)
                .unwrap()
                .extract::<Vec<PyMaybeRelocatable>>(py)
                .unwrap();

            assert_eq!(
                range,
                vec![
                    RelocatableValue(PyRelocatable {
                        segment_index: 2,
                        offset: 0
                    }),
                    RelocatableValue(PyRelocatable {
                        segment_index: 3,
                        offset: 0
                    })
                ]
            );
        });
    }

    #[test]
    fn get_range_with_gap() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );

            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }

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
                .insert_value(&Relocatable::from((1, 2)), &Relocatable::from((3, 0)))
                .unwrap();

            let maybe_relocatable = MaybeRelocatable::from((1, 0));
            let size = 2;
            let memory = PyMemory::new(&vm);

            let range = memory
                .get_range(maybe_relocatable.into(), size, py)
                .map_err(|err| to_vm_error(err, py));

            let expected_error = VirtualMachineError::CustomHint(String::from(
                "TypeError('Failed to call get_range method from Cairo memory')",
            ));
            assert!(range.is_err());
            assert_eq!(range.unwrap_err(), expected_error);
        });
    }

    // Test that get_range_as_ints() works as intended.
    #[test]
    fn get_range_as_ints() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let memory = PyMemory::new(&vm);

        let addr = {
            let mut vm = vm.vm.borrow_mut();
            let addr = vm.add_memory_segment();

            vm.load_data(
                &MaybeRelocatable::from(&addr),
                vec![
                    bigint!(1).into(),
                    bigint!(2).into(),
                    bigint!(3).into(),
                    bigint!(4).into(),
                ],
            )
            .expect("memory insertion failed");

            addr
        };

        assert_eq!(
            memory
                .get_range_as_ints(addr.into(), 4)
                .expect("get_range_as_ints() failed"),
            vec![bigint!(1), bigint!(2), bigint!(3), bigint!(4)],
        );
    }

    // Test that get_range_as_ints() fails when not all values are integers.
    #[test]
    fn get_range_as_ints_mixed() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let memory = PyMemory::new(&vm);

        let addr = {
            let mut vm = vm.vm.borrow_mut();
            let addr = vm.add_memory_segment();

            vm.load_data(
                &MaybeRelocatable::from(&addr),
                vec![
                    bigint!(1).into(),
                    bigint!(2).into(),
                    MaybeRelocatable::RelocatableValue((1, 2).into()),
                    bigint!(4).into(),
                ],
            )
            .expect("memory insertion failed");

            addr
        };

        memory
            .get_range_as_ints(addr.into(), 4)
            .expect_err("get_range_as_ints() succeeded (should have failed)");
    }

    // Test that get_range_as_ints() fails when the requested range is larger than the available
    // segments.
    #[test]
    fn get_range_as_ints_incomplete() {
        let vm = PyVM::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
            Vec::new(),
        );
        let memory = PyMemory::new(&vm);

        let addr = {
            let mut vm = vm.vm.borrow_mut();
            let addr = vm.add_memory_segment();

            vm.load_data(
                &MaybeRelocatable::from(&addr),
                vec![
                    bigint!(1).into(),
                    bigint!(2).into(),
                    bigint!(3).into(),
                    bigint!(4).into(),
                ],
            )
            .expect("memory insertion failed");

            addr
        };

        memory
            .get_range_as_ints(addr.into(), 8)
            .expect_err("get_range_as_ints() succeeded (should have failed)");
    }
}
