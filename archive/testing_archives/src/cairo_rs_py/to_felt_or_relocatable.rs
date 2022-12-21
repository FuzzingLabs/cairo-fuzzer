use num_bigint::BigInt;
use pyo3::prelude::*;

use crate::cairo_rs_py::relocatable::{PyMaybeRelocatable, PyRelocatable};

#[pyclass]
pub struct ToFeltOrRelocatableFunc;

#[pymethods]
impl ToFeltOrRelocatableFunc {
    pub fn __call__(&self, any: PyObject, py: Python) -> PyResult<PyObject> {
        match any.extract::<PyRelocatable>(py) {
            Ok(rel) => Ok(Into::<PyMaybeRelocatable>::into(rel).to_object(py)),
            Err(_) => Ok(Into::<PyMaybeRelocatable>::into(
                any.call_method0(py, "__int__")?.extract::<BigInt>(py)?,
            )
            .to_object(py)),
        }
    }
}
