use std::collections::HashMap;

use cairo_rs::{
    types::relocatable::Relocatable,
    vm::{errors::vm_errors::VirtualMachineError, runners::builtin_runner::SignatureBuiltinRunner},
};

use num_bigint::BigInt;
use pyo3::prelude::*;

use crate::cairo_rs_py::relocatable::PyRelocatable;

#[pyclass(name = "Signature")]
#[derive(Clone, Debug)]
pub struct PySignature {
    signatures: HashMap<PyRelocatable, (BigInt, BigInt)>,
}

#[pymethods]
impl PySignature {
    #[new]
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
        }
    }

    pub fn add_signature(&mut self, address: PyRelocatable, pair: (BigInt, BigInt)) {
        self.signatures.insert(address, pair);
    }
}

impl PySignature {
    pub fn update_signature(
        &self,
        signature_builtin: &mut SignatureBuiltinRunner,
    ) -> Result<(), VirtualMachineError> {
        for (address, pair) in self.signatures.iter() {
            signature_builtin
                .add_signature(Relocatable::from(address), pair)
                .map_err(VirtualMachineError::MemoryError)?
        }
        Ok(())
    }
}

impl Default for PySignature {
    fn default() -> Self {
        Self::new()
    }
}

impl ToPyObject for PySignature {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone().into_py(py)
    }
}
