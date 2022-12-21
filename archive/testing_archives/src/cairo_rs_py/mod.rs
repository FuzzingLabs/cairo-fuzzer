pub mod cairo_run;
pub mod cairo_runner;
mod ecdsa;
pub mod ids;
mod memory;
mod memory_segments;
mod range_check;
mod relocatable;
mod run_context;
mod scope_manager;
mod to_felt_or_relocatable;
mod utils;
mod vm_core;

use cairo_runner::PyCairoRunner;
use pyo3::prelude::*;

#[pymodule]
fn cairo_rs_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyCairoRunner>()?;
    Ok(())
}
