use crate::cairo_rs_py::relocatable::PyRelocatable;
use cairo_rs::types::relocatable::Relocatable;
use pyo3::{pyclass, pymethods};

#[pyclass]
pub struct PyRunContext {
    pc: Relocatable,
    ap: Relocatable,
    fp: Relocatable,
}

impl PyRunContext {
    pub fn new(pc: Relocatable, ap: Relocatable, fp: Relocatable) -> Self {
        Self { pc, ap, fp }
    }
}

#[pymethods]
impl PyRunContext {
    #[getter]
    pub fn pc(&self) -> PyRelocatable {
        self.pc.clone().into()
    }

    #[getter]
    pub fn ap(&self) -> PyRelocatable {
        self.ap.clone().into()
    }

    #[getter]
    pub fn fp(&self) -> PyRelocatable {
        self.fp.clone().into()
    }
}

#[cfg(test)]
mod test {
    use crate::run_context::PyRunContext;

    #[test]
    fn test_properties() {
        let run_context = PyRunContext::new((1, 2).into(), (3, 4).into(), (5, 6).into());

        assert_eq!(run_context.pc(), (1, 2).into());
        assert_eq!(run_context.ap(), (3, 4).into());
        assert_eq!(run_context.fp(), (5, 6).into());
    }
}
