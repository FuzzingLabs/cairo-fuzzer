use cairo_rs::vm::{
    errors::vm_errors::VirtualMachineError, runners::builtin_runner::RangeCheckBuiltinRunner,
};

use num_bigint::BigInt;
use pyo3::prelude::*;

#[pyclass(name = "RangeCheck")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyRangeCheck {
    #[pyo3(get)]
    bound: BigInt,
}

#[pymethods]
impl PyRangeCheck {
    #[new]
    pub fn new(value: BigInt) -> Self {
        Self { bound: value }
    }

    pub fn __repr__(&self) -> String {
        format!("Bound: {}", self.bound)
    }
}

impl From<Result<&RangeCheckBuiltinRunner, VirtualMachineError>> for PyRangeCheck {
    fn from(val: Result<&RangeCheckBuiltinRunner, VirtualMachineError>) -> Self {
        match val {
            Ok(range_check_builtin) => PyRangeCheck::from(range_check_builtin),
            Err(_err) => PyRangeCheck::new(BigInt::from(0)),
        }
    }
}

impl From<&RangeCheckBuiltinRunner> for PyRangeCheck {
    fn from(val: &RangeCheckBuiltinRunner) -> Self {
        Self {
            bound: val._bound.clone(),
        }
    }
}

impl ToPyObject for PyRangeCheck {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone().into_py(py)
    }
}

#[cfg(test)]
mod test {
    use super::PyRangeCheck;
    use super::*;
    use cairo_rs::{
        bigint,
        vm::{
            errors::vm_errors::VirtualMachineError,
            runners::builtin_runner::RangeCheckBuiltinRunner,
        },
    };
    use num_bigint::BigInt;
    use pyo3::ToPyObject;

    #[test]
    fn py_range_check_new() {
        let value = bigint!(12);
        let new_py_range_check = PyRangeCheck::new(value.clone());

        assert_eq!(new_py_range_check, PyRangeCheck { bound: value });
    }

    #[test]
    fn py_range_check_repr() {
        let value = bigint!(12);
        let new_py_range_check = PyRangeCheck::new(value);

        assert_eq!(new_py_range_check.__repr__(), String::from("Bound: 12"));
    }

    #[test]
    fn py_range_check_from_result_ok() {
        let value = 12;
        let bound = bigint!(1usize << 16).pow(value);
        let range_check_builtin = RangeCheckBuiltinRunner::new(value, value, true);
        let result_with_range_check_builtin: Result<&RangeCheckBuiltinRunner, VirtualMachineError> =
            Ok(&range_check_builtin);

        assert_eq!(
            PyRangeCheck::from(result_with_range_check_builtin),
            PyRangeCheck::new(bound)
        );
    }

    #[test]
    fn py_range_check_from_result_err() {
        let result_with_range_check_builtin: Result<&RangeCheckBuiltinRunner, VirtualMachineError> =
            Err(VirtualMachineError::BigintToU32Fail);

        assert_eq!(
            PyRangeCheck::from(result_with_range_check_builtin),
            PyRangeCheck::new(BigInt::from(0))
        );
    }

    #[test]
    fn py_range_check_from_range_check_builtin_runner() {
        let value = 12;
        let bound = bigint!(1usize << 16).pow(value);
        let range_check_builtin = RangeCheckBuiltinRunner::new(value, value, true);

        assert_eq!(
            PyRangeCheck::from(&range_check_builtin),
            PyRangeCheck::new(bound)
        );
    }

    #[test]
    fn py_range_check_to_py_object() {
        let value = bigint!(12);
        let new_py_range_check = PyRangeCheck::new(value.clone());

        Python::with_gil(|py| {
            let py_object = new_py_range_check
                .to_object(py)
                .extract::<PyRangeCheck>(py)
                .unwrap();

            assert_eq!(py_object, PyRangeCheck::new(value));
        });
    }
}
