use cairo_rs::vm::errors::vm_errors::VirtualMachineError;
use num_bigint::BigInt;
use pyo3::{PyErr, Python};
use std::{collections::HashMap, fmt::Display};

pyo3::import_exception!(starkware.cairo.lang.vm.vm_exceptions, VmException);

#[macro_export]
macro_rules! pycell {
    ($py:expr, $val:expr) => {
        PyCell::new($py, $val).map_err(|err| to_vm_error(err, $py))?
    };
}

pub fn to_vm_error(pyerror: PyErr, py: Python) -> VirtualMachineError {
    let value = pyerror.value(py);
    VirtualMachineError::CustomHint(format!("{:?}", value))
}

pub fn to_py_error<T: Display>(error: T) -> PyErr {
    // these are some dummy values, the only important one is
    // the `[error.to_string()]` one that lets the error message
    // from a hint to be printed (needed for some tests to pass)
    VmException::new_err((
        None::<i32>,
        None::<i32>,
        None::<i32>,
        None::<i32>,
        None::<i32>,
        [error.to_string()],
    ))
}

pub fn const_path_to_const_name(constants: &HashMap<String, BigInt>) -> HashMap<String, BigInt> {
    constants
        .iter()
        .map(|(name, value)| {
            let name = name.rsplit('.').next().unwrap_or(name);
            (name.to_string(), value.clone())
        })
        .collect()
}
