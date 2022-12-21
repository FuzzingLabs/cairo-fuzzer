use cairo_rs::{
    bigint,
    hint_processor::hint_processor_utils::bigint_to_usize,
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::errors::vm_errors::VirtualMachineError,
};
use num_bigint::BigInt;
use pyo3::{exceptions::PyArithmeticError, prelude::*, pyclass::CompareOp};

use crate::cairo_rs_py::utils::to_py_error;

const PYRELOCATABLE_COMPARE_ERROR: &str = "Cannot compare Relocatables of different segments";

#[derive(FromPyObject, Debug, Clone, PartialEq, Eq)]
pub enum PyMaybeRelocatable {
    Int(BigInt),
    RelocatableValue(PyRelocatable),
}

#[pyclass(name = "Relocatable")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PyRelocatable {
    #[pyo3(get)]
    pub segment_index: isize,
    #[pyo3(get)]
    pub offset: usize,
}

#[pymethods]
impl PyRelocatable {
    #[new]
    pub fn new(tuple: (isize, usize)) -> PyRelocatable {
        PyRelocatable {
            segment_index: tuple.0,
            offset: tuple.1,
        }
    }

    pub fn __add__(&self, value: usize) -> PyRelocatable {
        PyRelocatable {
            segment_index: self.segment_index,
            offset: self.offset + value,
        }
    }

    pub fn __sub__(&self, value: PyMaybeRelocatable, py: Python) -> PyResult<PyObject> {
        match value {
            PyMaybeRelocatable::Int(value) => {
                Ok(PyMaybeRelocatable::RelocatableValue(PyRelocatable {
                    segment_index: self.segment_index,
                    offset: self.offset - bigint_to_usize(&value).unwrap(),
                })
                .to_object(py))
            }
            PyMaybeRelocatable::RelocatableValue(address) => {
                if self.segment_index != address.segment_index {
                    return Err(VirtualMachineError::DiffIndexSub).map_err(to_py_error)?;
                }
                Ok(PyMaybeRelocatable::Int(bigint!(self.offset - address.offset)).to_object(py))
            }
        }
    }

    pub fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => {
                if self.segment_index == other.segment_index {
                    Ok(self.offset < other.offset)
                } else {
                    Err(PyArithmeticError::new_err(PYRELOCATABLE_COMPARE_ERROR))
                }
            }
            CompareOp::Le => {
                if self.segment_index == other.segment_index {
                    Ok(self.offset <= other.offset)
                } else {
                    Err(PyArithmeticError::new_err(PYRELOCATABLE_COMPARE_ERROR))
                }
            }
            CompareOp::Eq => {
                Ok((self.segment_index, self.offset) == (other.segment_index, other.offset))
            }
            CompareOp::Ne => {
                Ok((self.segment_index, self.offset) != (other.segment_index, other.offset))
            }
            CompareOp::Gt => {
                if self.segment_index == other.segment_index {
                    Ok(self.offset > other.offset)
                } else {
                    Err(PyArithmeticError::new_err(PYRELOCATABLE_COMPARE_ERROR))
                }
            }
            CompareOp::Ge => {
                if self.segment_index == other.segment_index {
                    Ok(self.offset >= other.offset)
                } else {
                    Err(PyArithmeticError::new_err(PYRELOCATABLE_COMPARE_ERROR))
                }
            }
        }
    }

    pub fn __repr__(&self) -> String {
        format!("({}, {})", self.segment_index, self.offset)
    }
}

impl From<PyMaybeRelocatable> for MaybeRelocatable {
    fn from(val: PyMaybeRelocatable) -> Self {
        match val {
            PyMaybeRelocatable::RelocatableValue(rel) => MaybeRelocatable::RelocatableValue(
                Relocatable::from((rel.segment_index, rel.offset)),
            ),
            PyMaybeRelocatable::Int(num) => MaybeRelocatable::Int(num),
        }
    }
}

impl From<&PyMaybeRelocatable> for MaybeRelocatable {
    fn from(val: &PyMaybeRelocatable) -> Self {
        match val {
            PyMaybeRelocatable::RelocatableValue(rel) => MaybeRelocatable::RelocatableValue(
                Relocatable::from((rel.segment_index, rel.offset)),
            ),
            PyMaybeRelocatable::Int(num) => MaybeRelocatable::Int(num.clone()),
        }
    }
}

impl From<MaybeRelocatable> for PyMaybeRelocatable {
    fn from(val: MaybeRelocatable) -> Self {
        match val {
            MaybeRelocatable::RelocatableValue(rel) => PyMaybeRelocatable::RelocatableValue(
                PyRelocatable::new((rel.segment_index, rel.offset)),
            ),
            MaybeRelocatable::Int(num) => PyMaybeRelocatable::Int(num),
        }
    }
}

impl From<&MaybeRelocatable> for PyMaybeRelocatable {
    fn from(val: &MaybeRelocatable) -> Self {
        match val {
            MaybeRelocatable::RelocatableValue(rel) => PyMaybeRelocatable::RelocatableValue(
                PyRelocatable::new((rel.segment_index, rel.offset)),
            ),
            MaybeRelocatable::Int(num) => PyMaybeRelocatable::Int(num.clone()),
        }
    }
}

impl ToPyObject for PyMaybeRelocatable {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            PyMaybeRelocatable::RelocatableValue(address) => address.clone().into_py(py),
            PyMaybeRelocatable::Int(value) => value.clone().into_py(py),
        }
    }
}

impl From<Relocatable> for PyRelocatable {
    fn from(val: Relocatable) -> Self {
        PyRelocatable::new((val.segment_index, val.offset))
    }
}

impl From<&PyRelocatable> for Relocatable {
    fn from(val: &PyRelocatable) -> Self {
        Relocatable::from((val.segment_index, val.offset))
    }
}

impl From<(isize, usize)> for PyRelocatable {
    fn from(val: (isize, usize)) -> Self {
        PyRelocatable::new((val.0, val.1))
    }
}

impl From<Relocatable> for PyMaybeRelocatable {
    fn from(val: Relocatable) -> Self {
        PyMaybeRelocatable::RelocatableValue(val.into())
    }
}

impl From<PyRelocatable> for PyMaybeRelocatable {
    fn from(val: PyRelocatable) -> Self {
        PyMaybeRelocatable::RelocatableValue(val)
    }
}

impl From<&BigInt> for PyMaybeRelocatable {
    fn from(val: &BigInt) -> Self {
        PyMaybeRelocatable::Int(val.clone())
    }
}

impl From<BigInt> for PyMaybeRelocatable {
    fn from(val: BigInt) -> Self {
        PyMaybeRelocatable::Int(val)
    }
}

#[cfg(test)]
mod test {
    use cairo_rs::{bigint, types::relocatable::MaybeRelocatable};
    use num_bigint::BigInt;
    use pyo3::ToPyObject;
    use pyo3::{pyclass::CompareOp, Python};

    use crate::relocatable::Relocatable;
    use crate::relocatable::{PyMaybeRelocatable, PyRelocatable};

    #[test]
    fn py_relocatable_new() {
        let values = (1, 2);

        let py_relocatable = PyRelocatable::new(values);

        assert_eq!(
            py_relocatable,
            PyRelocatable {
                segment_index: values.0,
                offset: values.1,
            }
        );
    }

    #[test]
    fn py_relocatable_repr() {
        let values = (1, 2);

        let py_relocatable = PyRelocatable::new(values);

        assert_eq!(
            py_relocatable.__repr__(),
            format!(
                "({}, {})",
                py_relocatable.segment_index, py_relocatable.offset
            )
        );
    }

    #[test]
    fn py_relocatable_add() {
        let values = (1, 2);

        let py_relocatable = PyRelocatable::new(values);

        assert_eq!(py_relocatable.__add__(2), PyRelocatable::new((1, 4)));
    }

    #[test]
    fn py_relocatable_sub_with_int() {
        Python::with_gil(|py| {
            let values = (1, 2);

            let py_relocatable = PyRelocatable::new(values);
            let bigint_value = bigint!(1);
            let py_maybe_relocatable_int_variant = PyMaybeRelocatable::Int(bigint_value);
            let substraction = py_relocatable
                .__sub__(py_maybe_relocatable_int_variant, py)
                .unwrap()
                .extract::<PyMaybeRelocatable>(py)
                .unwrap();

            assert_eq!(
                substraction,
                PyMaybeRelocatable::RelocatableValue(PyRelocatable {
                    segment_index: 1,
                    offset: 1,
                })
            );
        });
    }

    #[test]
    fn py_relocatable_sub_with_relocatable_value() {
        Python::with_gil(|py| {
            let values1 = (2, 5);
            let values2 = (2, 4);

            let py_relocatable1 = PyRelocatable::new(values1);
            let py_relocatable2 = PyRelocatable::new(values2);

            let py_maybe_relocatable = PyMaybeRelocatable::RelocatableValue(py_relocatable2);
            let substraction = py_relocatable1
                .__sub__(py_maybe_relocatable, py)
                .unwrap()
                .extract::<PyMaybeRelocatable>(py)
                .unwrap();

            assert_eq!(substraction, PyMaybeRelocatable::Int(bigint!(1)));
        });
    }

    #[test]
    fn py_relocatable_sub_with_relocatable_value_err() {
        Python::with_gil(|py| {
            let values1 = (12, 5);
            let values2 = (2, 4);

            let py_relocatable1 = PyRelocatable::new(values1);
            let py_relocatable2 = PyRelocatable::new(values2);

            let py_maybe_relocatable = PyMaybeRelocatable::RelocatableValue(py_relocatable2);
            assert!(py_relocatable1.__sub__(py_maybe_relocatable, py).is_err());
        });
    }

    #[test]
    fn py_relocatable_richcmp_valid() {
        let values1 = (2, 5);
        let values2 = (2, 4);

        let py_relocatable1 = PyRelocatable::new(values1);
        let py_relocatable2 = PyRelocatable::new(values2);

        assert!(!py_relocatable1
            .__richcmp__(&py_relocatable2, CompareOp::Eq)
            .unwrap());
        assert!(py_relocatable1
            .__richcmp__(&py_relocatable2, CompareOp::Ge)
            .unwrap());
        assert!(py_relocatable1
            .__richcmp__(&py_relocatable2, CompareOp::Gt)
            .unwrap());
        assert!(!py_relocatable1
            .__richcmp__(&py_relocatable2, CompareOp::Le)
            .unwrap());
        assert!(!py_relocatable1
            .__richcmp__(&py_relocatable2, CompareOp::Lt)
            .unwrap());
        assert!(py_relocatable1
            .__richcmp__(&py_relocatable2, CompareOp::Ne)
            .unwrap());
    }

    #[test]
    fn py_relocatable_richcmp_error() {
        let values1 = (1, 5);
        let values2 = (2, 4);

        let py_relocatable1 = PyRelocatable::new(values1);
        let py_relocatable2 = PyRelocatable::new(values2);

        assert!(py_relocatable1
            .__richcmp__(&py_relocatable2, CompareOp::Ge)
            .is_err());
        assert!(py_relocatable1
            .__richcmp__(&py_relocatable2, CompareOp::Gt)
            .is_err());
        assert!(py_relocatable1
            .__richcmp__(&py_relocatable2, CompareOp::Le)
            .is_err());
        assert!(py_relocatable1
            .__richcmp__(&py_relocatable2, CompareOp::Lt)
            .is_err());
    }

    #[test]
    fn maybe_relocatable_from_py_maybe_relocatable() {
        let py_maybe_relocatable_int = PyMaybeRelocatable::Int(bigint!(1));
        let py_relocatable = PyRelocatable::new((1, 1));
        let py_maybe_relocatable_relocatable = PyMaybeRelocatable::RelocatableValue(py_relocatable);

        assert_eq!(
            MaybeRelocatable::from(py_maybe_relocatable_int),
            MaybeRelocatable::Int(bigint!(1))
        );
        assert_eq!(
            MaybeRelocatable::from(py_maybe_relocatable_relocatable),
            MaybeRelocatable::RelocatableValue(Relocatable::from((1, 1)))
        );
    }

    #[test]
    fn maybe_relocatable_from_py_maybe_relocatable_ref() {
        let py_maybe_relocatable_int = PyMaybeRelocatable::Int(bigint!(1));
        let py_relocatable = PyRelocatable::new((1, 1));
        let py_maybe_relocatable_relocatable = PyMaybeRelocatable::RelocatableValue(py_relocatable);

        assert_eq!(
            MaybeRelocatable::from(&py_maybe_relocatable_int),
            MaybeRelocatable::Int(bigint!(1))
        );
        assert_eq!(
            MaybeRelocatable::from(&py_maybe_relocatable_relocatable),
            MaybeRelocatable::RelocatableValue(Relocatable::from((1, 1)))
        );
    }

    #[test]
    fn py_maybe_relocatable_from_maybe_relocatable() {
        let maybe_relocatable_int = MaybeRelocatable::Int(bigint!(1));
        let maybe_relocatable_reloc = MaybeRelocatable::RelocatableValue(Relocatable {
            segment_index: 1,
            offset: 1,
        });

        assert_eq!(
            PyMaybeRelocatable::from(maybe_relocatable_int),
            PyMaybeRelocatable::Int(bigint!(1))
        );

        assert_eq!(
            PyMaybeRelocatable::from(maybe_relocatable_reloc),
            PyMaybeRelocatable::RelocatableValue(PyRelocatable {
                segment_index: 1,
                offset: 1
            })
        );
    }

    #[test]
    fn py_maybe_relocatable_from_maybe_relocatable_ref() {
        let maybe_relocatable_int = MaybeRelocatable::Int(bigint!(1));
        let maybe_relocatable_reloc = MaybeRelocatable::RelocatableValue(Relocatable {
            segment_index: 1,
            offset: 1,
        });

        assert_eq!(
            PyMaybeRelocatable::from(&maybe_relocatable_int),
            PyMaybeRelocatable::Int(bigint!(1))
        );

        assert_eq!(
            PyMaybeRelocatable::from(&maybe_relocatable_reloc),
            PyMaybeRelocatable::RelocatableValue(PyRelocatable {
                segment_index: 1,
                offset: 1
            })
        );
    }

    #[test]
    fn py_relocatable_from_relocatable() {
        let relocatable = Relocatable {
            segment_index: 32,
            offset: 12,
        };

        assert_eq!(
            PyRelocatable::from(relocatable),
            PyRelocatable {
                segment_index: 32,
                offset: 12,
            }
        );
    }

    #[test]
    fn relocatable_from_py_relocatable() {
        let relocatable = PyRelocatable {
            segment_index: 32,
            offset: 12,
        };

        assert_eq!(
            Relocatable::from(&relocatable),
            Relocatable {
                segment_index: 32,
                offset: 12,
            }
        );
    }

    #[test]
    fn py_relocatable_from_tuple() {
        let values: (isize, usize) = (123, 456);

        assert_eq!(
            PyRelocatable::from(values),
            PyRelocatable {
                segment_index: 123,
                offset: 456,
            }
        );
    }

    #[test]
    fn py_maybe_relocatable_from_relocatable() {
        let relocatable = Relocatable {
            segment_index: 32,
            offset: 12,
        };

        assert_eq!(
            PyMaybeRelocatable::from(relocatable),
            PyMaybeRelocatable::RelocatableValue(PyRelocatable {
                segment_index: 32,
                offset: 12,
            })
        );
    }

    #[test]
    fn py_maybe_relocatable_from_py_relocatable() {
        let relocatable = PyRelocatable {
            segment_index: 32,
            offset: 12,
        };

        assert_eq!(
            PyMaybeRelocatable::from(relocatable),
            PyMaybeRelocatable::RelocatableValue(PyRelocatable {
                segment_index: 32,
                offset: 12,
            })
        );
    }

    #[test]
    fn py_maybe_relocatable_from_bigint() {
        let value = bigint!(7654321);

        assert_eq!(
            PyMaybeRelocatable::from(value.clone()),
            PyMaybeRelocatable::Int(value)
        );
    }

    #[test]
    fn py_maybe_relocatable_from_bigint_ref() {
        let value = bigint!(7654321);

        assert_eq!(
            PyMaybeRelocatable::from(&value),
            PyMaybeRelocatable::Int(value)
        );
    }

    #[test]
    fn py_maybe_relocatable_to_object() {
        let py_maybe_relocatable_int = PyMaybeRelocatable::Int(bigint!(6543));
        let py_maybe_relocatable_reloc = PyMaybeRelocatable::RelocatableValue(PyRelocatable {
            segment_index: 43,
            offset: 123,
        });

        Python::with_gil(|py| {
            let py_object_int = py_maybe_relocatable_int
                .to_object(py)
                .extract::<PyMaybeRelocatable>(py)
                .unwrap();

            assert_eq!(py_object_int, PyMaybeRelocatable::Int(bigint!(6543)));

            let py_object_reloc = py_maybe_relocatable_reloc
                .to_object(py)
                .extract::<PyMaybeRelocatable>(py)
                .unwrap();

            assert_eq!(
                py_object_reloc,
                PyMaybeRelocatable::RelocatableValue(PyRelocatable {
                    segment_index: 43,
                    offset: 123,
                })
            );
        })
    }
}
