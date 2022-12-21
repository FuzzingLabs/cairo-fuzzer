use crate::cairo_rs_py::utils::const_path_to_const_name;
use num_bigint::BigInt;
use pyo3::exceptions::PyValueError;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use cairo_rs::{
    hint_processor::{
        hint_processor_definition::HintReference, hint_processor_utils::bigint_to_usize,
    },
    serde::deserialize_program::{ApTracking, Member},
    types::{
        instruction::Register,
        relocatable::{MaybeRelocatable, Relocatable},
    },
    vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use pyo3::{
    exceptions::PyAttributeError, pyclass, pymethods, IntoPy, PyObject, PyResult, Python,
    ToPyObject,
};

use crate::{cairo_rs_py::relocatable::PyMaybeRelocatable, cairo_rs_py::vm_core::PyVM};

const IDS_GET_ERROR_MSG: &str = "Failed to get ids value";
const IDS_SET_ERROR_MSG: &str = "Failed to set ids value to Cairo memory";
const STRUCT_TYPES_GET_ERROR_MSG: &str = "Failed to get struct type";

#[pyclass(unsendable)]
pub struct PyIds {
    vm: Rc<RefCell<VirtualMachine>>,
    references: HashMap<String, HintReference>,
    ap_tracking: ApTracking,
    constants: HashMap<String, BigInt>,
    struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
}

#[pymethods]
impl PyIds {
    #[getter]
    pub fn __getattr__(&self, name: &str, py: Python) -> PyResult<PyObject> {
        if let Some(constant) = self.constants.get(name) {
            return Ok(constant.to_object(py));
        }

        // Support for for ids.{Struct Definition} information
        // Example: ids.DictAccess
        let mut types_set = HashSet::new();
        for key in self.struct_types.keys() {
            types_set.insert(
                key.rsplit('.')
                    .next()
                    .ok_or_else(|| PyValueError::new_err(STRUCT_TYPES_GET_ERROR_MSG))?,
            );
        }
        if types_set.contains(name) {
            let mut structs_size = HashMap::new();

            for (key, v) in self.struct_types.iter() {
                let max_member = v.values().max_by(|x, y| x.offset.cmp(&y.offset));

                let max_offset = match max_member {
                    Some(member) => member.offset + 1,
                    _ => 0,
                };
                structs_size.insert(
                    key.rsplit('.')
                        .next()
                        .ok_or_else(|| PyValueError::new_err(STRUCT_TYPES_GET_ERROR_MSG))?,
                    max_offset,
                );
            }
            if let Some(size) = structs_size.get(name) {
                return Ok(CairoStruct { SIZE: *size }.into_py(py));
            }
        }

        let hint_ref = self
            .references
            .get(name)
            .ok_or_else(|| PyValueError::new_err(IDS_GET_ERROR_MSG))?;

        if let Some(cairo_type) = hint_ref.cairo_type.as_deref() {
            let chars = cairo_type.chars().rev();
            let clear_ref = chars
                .skip_while(|c| c == &'*')
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>();

            if self.struct_types.contains_key(cairo_type) {
                return Ok(PyTypedId {
                    vm: self.vm.clone(),
                    hint_value: compute_addr_from_reference(
                        hint_ref,
                        &self.vm.borrow(),
                        &self.ap_tracking,
                    )?,
                    cairo_type: cairo_type.to_string(),
                    struct_types: Rc::clone(&self.struct_types),
                }
                .into_py(py));
            } else if self.struct_types.contains_key(&clear_ref) {
                let addr =
                    compute_addr_from_reference(hint_ref, &self.vm.borrow(), &self.ap_tracking)?;

                let hint_value = self
                    .vm
                    .borrow()
                    .get_relocatable(&addr)
                    .map_err(|err| PyValueError::new_err(err.to_string()))?
                    .into_owned();

                return Ok(PyTypedId {
                    vm: self.vm.clone(),
                    hint_value,
                    cairo_type: clear_ref,
                    struct_types: Rc::clone(&self.struct_types),
                }
                .into_py(py));
            }
        }

        Ok(
            get_value_from_reference(&self.vm.borrow(), hint_ref, &self.ap_tracking)?
                .to_object(py)
                .into_py(py),
        )
    }

    pub fn __setattr__(&self, name: &str, val: PyMaybeRelocatable) -> PyResult<()> {
        let hint_ref = self
            .references
            .get(name)
            .ok_or_else(|| PyValueError::new_err(IDS_SET_ERROR_MSG))?;
        let var_addr = compute_addr_from_reference(hint_ref, &self.vm.borrow(), &self.ap_tracking)?;
        self.vm
            .borrow_mut()
            .insert_value(&var_addr, &val)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }
}

impl PyIds {
    pub fn new(
        vm: &PyVM,
        references: &HashMap<String, HintReference>,
        ap_tracking: &ApTracking,
        constants: &HashMap<String, BigInt>,
        struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
    ) -> PyIds {
        PyIds {
            vm: vm.get_vm(),
            references: references.clone(),
            ap_tracking: ap_tracking.clone(),
            constants: const_path_to_const_name(constants),
            struct_types,
        }
    }
}

#[allow(non_snake_case)]
#[pyclass(unsendable)]
struct CairoStruct {
    #[pyo3(get)]
    SIZE: usize,
}

#[pyclass(unsendable)]
struct PyTypedId {
    vm: Rc<RefCell<VirtualMachine>>,
    hint_value: Relocatable,
    cairo_type: String,
    struct_types: Rc<HashMap<String, HashMap<String, Member>>>,
}

#[pymethods]
impl PyTypedId {
    #[getter]
    fn __getattr__(&self, py: Python, name: &str) -> PyResult<PyObject> {
        if name == "address_" {
            return Ok(PyMaybeRelocatable::from(self.hint_value.clone()).to_object(py));
        }
        let struct_type = self.struct_types.get(&self.cairo_type).unwrap();

        match struct_type.get(name) {
            Some(member) => {
                let vm = self.vm.borrow();
                Ok(match member.cairo_type.as_str() {
                    "felt" | "felt*" => vm
                        .get_maybe(&(&self.hint_value + member.offset))
                        .map_err(|err| PyValueError::new_err(err.to_string()))?
                        .map(|x| PyMaybeRelocatable::from(x).to_object(py))
                        .unwrap_or_else(|| py.None()),

                    cairo_type => PyTypedId {
                        vm: self.vm.clone(),
                        hint_value: (&self.hint_value + member.offset),
                        cairo_type: cairo_type.to_string(),
                        struct_types: self.struct_types.clone(),
                    }
                    .into_py(py),
                })
            }
            None => Err(PyAttributeError::new_err(format!(
                "'PyTypeId' object has no attribute '{}'",
                name
            ))),
        }
    }

    pub fn __setattr__(&self, field_name: &str, val: PyMaybeRelocatable) -> PyResult<()> {
        let struct_type = self
            .struct_types
            .get(&self.cairo_type)
            .ok_or_else(|| PyValueError::new_err(STRUCT_TYPES_GET_ERROR_MSG))?;

        let member = struct_type.get(field_name).ok_or_else(|| {
            PyAttributeError::new_err(format!(
                "'PyTypeId' object has no attribute '{}'",
                field_name
            ))
        })?;

        let mut vm = self.vm.borrow_mut();
        match member.cairo_type.as_str() {
            "felt" | "felt*" => {
                let field_addr = &self.hint_value + member.offset;
                vm.insert_value(&field_addr, val).map_err(|err| PyValueError::new_err(err.to_string()))
            }

            _cairo_type => Err(PyValueError::new_err("Error: It should be possible to assign a struct into another struct's field. See issue #86")),
        }
    }
}

///Returns the Value given by a reference as an Option<MaybeRelocatable>
pub fn get_value_from_reference(
    vm: &VirtualMachine,
    hint_reference: &HintReference,
    ap_tracking: &ApTracking,
) -> PyResult<PyMaybeRelocatable> {
    //First handle case on only immediate
    if let (None, Some(num)) = (
        hint_reference.register.as_ref(),
        hint_reference.immediate.as_ref(),
    ) {
        return Ok(PyMaybeRelocatable::from(num));
    }
    //Then calculate address
    let var_addr = compute_addr_from_reference(hint_reference, vm, ap_tracking)?;
    let value = if hint_reference.dereference {
        vm.get_maybe(&var_addr)
            .map_err(|err| PyValueError::new_err(err.to_string()))?
    } else {
        return Ok(PyMaybeRelocatable::from(var_addr));
    };
    match value {
        Some(MaybeRelocatable::RelocatableValue(ref rel)) => {
            if let Some(immediate) = &hint_reference.immediate {
                let modified_value = rel
                    + bigint_to_usize(immediate)
                        .map_err(|err| PyValueError::new_err(err.to_string()))?;
                Ok(PyMaybeRelocatable::from(modified_value))
            } else {
                Ok(PyMaybeRelocatable::from(rel.clone()))
            }
        }
        Some(MaybeRelocatable::Int(ref num)) => Ok(PyMaybeRelocatable::Int(num.clone())),
        None => Err(PyValueError::new_err(
            VirtualMachineError::FailedToGetIds.to_string(),
        )),
    }
}

///Computes the memory address of the ids variable indicated by the HintReference as a Relocatable
pub fn compute_addr_from_reference(
    //Reference data of the ids variable
    hint_reference: &HintReference,
    vm: &VirtualMachine,
    //ApTracking of the Hint itself
    hint_ap_tracking: &ApTracking,
) -> PyResult<Relocatable> {
    let base_addr = match hint_reference.register {
        //This should never fail
        Some(Register::FP) => vm.get_fp(),
        Some(Register::AP) => {
            let var_ap_trackig = hint_reference.ap_tracking_data.as_ref().ok_or_else(|| {
                PyValueError::new_err(VirtualMachineError::NoneApTrackingData.to_string())
            })?;

            let ap = vm.get_ap();

            apply_ap_tracking_correction(&ap, var_ap_trackig, hint_ap_tracking)
                .map_err(|err| PyValueError::new_err(err.to_string()))?
        }
        None => {
            return Err(PyValueError::new_err(
                VirtualMachineError::NoRegisterInReference.to_string(),
            ))
        }
    };
    if hint_reference.offset1.is_negative()
        && base_addr.offset < hint_reference.offset1.unsigned_abs().try_into()?
    {
        return Err(PyValueError::new_err(
            VirtualMachineError::FailedToGetIds.to_string(),
        ));
    }
    if !hint_reference.inner_dereference {
        Ok(base_addr + hint_reference.offset1 + hint_reference.offset2)
    } else {
        let addr = base_addr + hint_reference.offset1;
        let dereferenced_addr = vm
            .get_relocatable(&addr)
            .map_err(|err| PyValueError::new_err(err.to_string()))?
            .into_owned();
        if let Some(imm) = &hint_reference.immediate {
            Ok(dereferenced_addr
                + bigint_to_usize(imm).map_err(|err| PyValueError::new_err(err.to_string()))?)
        } else {
            Ok(dereferenced_addr + hint_reference.offset2)
        }
    }
}

//TODO: Make this function public and import it from cairo-rs
fn apply_ap_tracking_correction(
    ap: &Relocatable,
    ref_ap_tracking: &ApTracking,
    hint_ap_tracking: &ApTracking,
) -> Result<Relocatable, VirtualMachineError> {
    // check that both groups are the same
    if ref_ap_tracking.group != hint_ap_tracking.group {
        return Err(VirtualMachineError::InvalidTrackingGroup(
            ref_ap_tracking.group,
            hint_ap_tracking.group,
        ));
    }
    let ap_diff = hint_ap_tracking.offset - ref_ap_tracking.offset;
    ap.sub(ap_diff)
}

#[cfg(test)]
mod tests {
    use cairo_rs::bigint;
    use num_bigint::{BigInt, Sign};
    use pyo3::{types::PyDict, PyCell};

    use crate::{cairo_rs_py::memory::PyMemory, cairo_rs_py::relocatable::PyRelocatable, cairo_rs_py::utils::to_vm_error};

    use super::*;

    fn create_simple_struct_type() -> (String, HashMap<String, Member>) {
        //Return new type for SimpleStruct { x: felt, ptr: felt* }
        (
            String::from("SimpleStruct"),
            HashMap::from([
                (
                    String::from("x"),
                    Member {
                        cairo_type: String::from("felt"),
                        offset: 0,
                    },
                ),
                (
                    String::from("ptr"),
                    Member {
                        cairo_type: String::from("felt*"),
                        offset: 1,
                    },
                ),
            ]),
        )
    }

    #[test]
    fn ids_get_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            //Create references
            let mut references = HashMap::new();
            references.insert(String::from("a"), HintReference::new_simple(1));

            //Create constants
            let mut constants = HashMap::new();
            constants.insert(String::from("CONST"), bigint!(3));

            //Insert ids.a into memory
            vm.vm
                .borrow_mut()
                .insert_value(
                    &Relocatable::from((1, 1)),
                    &MaybeRelocatable::from(bigint!(2)),
                )
                .unwrap();

            let memory = PyMemory::new(&vm);
            let fp = PyRelocatable::from((1, 0));
            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &constants,
                Rc::new(HashMap::new()),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = r#"
memory[fp] = ids.a
memory[fp+2] = ids.CONST
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(|err| to_vm_error(err, py)), Ok(()));
            //Check ids.a is now at memory[fp]
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 0))),
                Ok(Some(MaybeRelocatable::from(bigint!(2))))
            );
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 2))),
                Ok(Some(MaybeRelocatable::from(bigint!(3))))
            );
        });
    }

    #[test]
    fn ids_get_simple_struct() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            //Create references
            let mut references = HashMap::new();
            references.insert(
                String::from("a"),
                HintReference {
                    register: Some(Register::FP),
                    offset1: 0,
                    offset2: 0,
                    dereference: true,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: None,
                    cairo_type: Some(String::from("SimpleStruct")),
                },
            );

            //Create struct types
            let struct_types = HashMap::from([create_simple_struct_type()]);

            //Insert ids.a.x into memory
            vm.vm
                .borrow_mut()
                .insert_value(
                    &Relocatable::from((1, 0)),
                    &MaybeRelocatable::from(bigint!(55)),
                )
                .unwrap();

            //Insert ids.a.ptr into memory
            vm.vm
                .borrow_mut()
                .insert_value(&Relocatable::from((1, 1)), &MaybeRelocatable::from((1, 0)))
                .unwrap();

            let memory = PyMemory::new(&vm);
            let fp = PyRelocatable::from((1, 0));
            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(struct_types),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = r#"
memory[fp] = ids.a.x
memory[fp + 1] = ids.a.ptr
memory[fp + 2] = ids.SimpleStruct.SIZE
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(|err| to_vm_error(err, py)), Ok(()));
            //Check ids.a.x is now at memory[fp]
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 0))),
                Ok(Some(MaybeRelocatable::from(bigint!(55))))
            );
            //Check ids.a.ptr is now at memory[fp + 1]
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 1))),
                Ok(Some(MaybeRelocatable::from((1, 0))))
            );
            //Check ids.SimpleStruct.SIZE is now at memory[fp + 2]
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 2))),
                Ok(Some(MaybeRelocatable::from(bigint!(2))))
            );

            //ids.a.y does not exist
            let code = "ids.a.y";

            let py_result = py.run(code, Some(globals), None);

            assert!(py_result.is_err());
        });
    }

    #[test]
    fn ids_get_nested_struct() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            //Create references
            let mut references = HashMap::new();
            references.insert(
                String::from("ns"),
                HintReference {
                    register: Some(Register::FP),
                    offset1: 0,
                    offset2: 0,
                    dereference: true,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: None,
                    cairo_type: Some(String::from("NestedStruct")),
                },
            );

            //Create struct types
            let mut struct_types = HashMap::new();

            //Insert new type Struct {}
            struct_types.insert(String::from("Struct"), HashMap::new());

            //Insert new type NestedStruct { x: felt, y: felt }
            struct_types.insert(
                String::from("NestedStruct"),
                HashMap::from([
                    (
                        String::from("x"),
                        Member {
                            cairo_type: String::from("felt"),
                            offset: 0,
                        },
                    ),
                    (
                        String::from("struct"),
                        Member {
                            cairo_type: String::from("Struct"),
                            offset: 1,
                        },
                    ),
                ]),
            );

            //Insert ids.ns.x into memory
            vm.vm
                .borrow_mut()
                .insert_value(
                    &Relocatable::from((1, 0)),
                    &MaybeRelocatable::from(bigint!(55)),
                )
                .unwrap();

            //Insert ids.ns.ptr into memory
            vm.vm
                .borrow_mut()
                .insert_value(&Relocatable::from((1, 1)), &MaybeRelocatable::from((1, 0)))
                .unwrap();

            let memory = PyMemory::new(&vm);
            let fp = PyRelocatable::from((1, 3));
            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(struct_types),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = r#"
memory[fp] = ids.Struct.SIZE
memory[fp + 1] = ids.ns.struct.address_
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(|err| to_vm_error(err, py)), Ok(()));

            //Check ids.Struct.SIZE is now at memory[fp]
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 3))),
                Ok(Some(MaybeRelocatable::from(bigint!(0))))
            );
            //Check that address of ids.ns.struct is now at memory[fp + 1]
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 4))),
                Ok(Some(MaybeRelocatable::from((1, 1))))
            );
        });
    }

    #[test]
    fn ids_get_from_pointer() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..3 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            //Create references
            let mut references = HashMap::new();
            //Insert SimpleStruct pointer
            references.insert(
                String::from("ssp"),
                HintReference {
                    register: Some(Register::FP),
                    offset1: 0,
                    offset2: 0,
                    dereference: true,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: None,
                    cairo_type: Some(String::from("SimpleStruct*")),
                },
            );
            //Insert pointer with double dereference
            references.insert(
                String::from("ssp_x_ptr"),
                HintReference::new(0, 0, true, true),
            );

            //Insert ids.ssp into memory
            vm.vm
                .borrow_mut()
                .insert_value(&Relocatable::from((1, 0)), &MaybeRelocatable::from((2, 0)))
                .unwrap();

            let struct_types = HashMap::from([create_simple_struct_type()]);

            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(struct_types),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = r#"
ids.ssp.x = 5
assert ids.ssp.x == 5
assert ids.ssp_x_ptr == 5
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(|err| to_vm_error(err, py)), Ok(()));
        });
    }

    #[test]
    fn ids_failed_get_test() {
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
            let fp = PyRelocatable::from((1, 0));
            let ids = PyIds::new(
                &vm,
                &HashMap::new(),
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = r"memory[fp] = ids.b";

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(
                py_result.map_err(|err| to_vm_error(err, py)),
                Err(to_vm_error(PyValueError::new_err(IDS_GET_ERROR_MSG), py))
            );
        });
    }

    #[test]
    fn ids_set_test() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            //Create references
            let mut references = HashMap::new();
            references.insert(String::from("a"), HintReference::new_simple(1));

            //Create constants
            let mut constants = HashMap::new();
            constants.insert(String::from("CONST"), bigint!(3));

            vm.vm
                .borrow_mut()
                .insert_value(
                    &Relocatable::from((1, 0)),
                    &MaybeRelocatable::from(bigint!(2)),
                )
                .unwrap();

            let memory = PyMemory::new(&vm);
            let fp = PyRelocatable::from((1, 0));
            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &constants,
                Rc::new(HashMap::new()),
            );

            let globals = PyDict::new(py);

            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = "ids.a = memory[fp]";

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(|err| to_vm_error(err, py)), Ok(()));
            //Check ids.a now contains memory[fp]
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 1))),
                Ok(Some(MaybeRelocatable::from(bigint!(2))))
            );

            //ids.b does not exist
            let code = "ids.b = memory[fp]";

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(
                py_result.map_err(|err| to_vm_error(err, py)),
                Err(to_vm_error(PyValueError::new_err(IDS_SET_ERROR_MSG), py))
            );
        });
    }

    #[test]
    fn ids_set_struct_attribute() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            //Create references
            let mut references = HashMap::new();
            references.insert(
                String::from("struct"),
                HintReference {
                    register: Some(Register::FP),
                    offset1: 0,
                    offset2: 0,
                    dereference: true,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: None,
                    cairo_type: Some(String::from("SimpleStruct")),
                },
            );
            //Insert reference to fp's address
            references.insert(
                String::from("fp"),
                HintReference {
                    register: Some(Register::FP),
                    offset1: 0,
                    offset2: 0,
                    dereference: false,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: None,
                    cairo_type: None,
                },
            );

            let struct_types = HashMap::from([create_simple_struct_type()]);

            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(struct_types),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = r#"
ids.struct.x = 5

ids.struct.ptr = ids.fp
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(|err| to_vm_error(err, py)), Ok(()));
            //Check ids.struct.x now contains 5
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 0))),
                Ok(Some(MaybeRelocatable::from(bigint!(5))))
            );
            //Check ids.struct.x now contains fp's address
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 1))),
                Ok(Some(MaybeRelocatable::from(vm.get_vm().borrow().get_fp())))
            );

            //ids.struct.y does not exist
            let code = "ids.struct.y = 10";

            let py_result = py.run(code, Some(globals), None);

            assert!(py_result.is_err());
        });
    }

    #[test]
    fn ids_ap_tracked_ref() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            //Create references
            let mut references = HashMap::new();
            //Insert basic ap tracking reference
            references.insert(
                String::from("ok_ref"),
                HintReference {
                    register: Some(Register::AP),
                    offset1: 0,
                    offset2: 0,
                    dereference: true,
                    inner_dereference: false,
                    ap_tracking_data: Some(ApTracking::default()),
                    immediate: None,
                    cairo_type: None,
                },
            );
            //Insert ap tracking reference with non-matching group
            references.insert(
                String::from("bad_ref"),
                HintReference {
                    register: Some(Register::AP),
                    offset1: 0,
                    offset2: 0,
                    dereference: true,
                    inner_dereference: false,
                    ap_tracking_data: Some(ApTracking {
                        group: 1,
                        offset: 0,
                    }),
                    immediate: None,
                    cairo_type: None,
                },
            );
            //Insert ap tracking reference with no tracking
            references.insert(
                String::from("none_ref"),
                HintReference {
                    register: Some(Register::AP),
                    offset1: 0,
                    offset2: 0,
                    dereference: true,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: None,
                    cairo_type: None,
                },
            );

            let memory = PyMemory::new(&vm);
            let fp = PyRelocatable::from((1, 0));
            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = r#"
ids.ok_ref = 5
memory[fp] = ids.ok_ref
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(|err| to_vm_error(err, py)), Ok(()));
            //Check ids.a is now at memory[fp]
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 0))),
                Ok(Some(MaybeRelocatable::from(bigint!(5))))
            );

            let code = r"ids.bad_ref";

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(
                py_result.map_err(|err| to_vm_error(err, py)),
                Err(to_vm_error(
                    PyValueError::new_err(
                        VirtualMachineError::InvalidTrackingGroup(1, 0).to_string()
                    ),
                    py
                ))
            );

            let code = r"ids.none_ref";

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(
                py_result.map_err(|err| to_vm_error(err, py)),
                Err(to_vm_error(
                    PyValueError::new_err(VirtualMachineError::NoneApTrackingData.to_string()),
                    py
                ))
            );
        });
    }

    #[test]
    fn ids_no_register_ref() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            //Create references
            let mut references = HashMap::new();
            let imm = 89;
            //Insert no register reference with immediate value
            references.insert(
                String::from("imm_ref"),
                HintReference {
                    register: None,
                    offset1: 0,
                    offset2: 0,
                    dereference: true,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: Some(bigint!(imm)),
                    cairo_type: None,
                },
            );
            //Insert no register reference without imm
            references.insert(
                String::from("no_reg_ref"),
                HintReference {
                    register: None,
                    offset1: 0,
                    offset2: 0,
                    dereference: true,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: None,
                    cairo_type: None,
                },
            );

            let memory = PyMemory::new(&vm);
            let fp = PyRelocatable::from((1, 0));
            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = r"memory[fp] = ids.imm_ref";

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(|err| to_vm_error(err, py)), Ok(()));
            //Check ids.a is now at memory[fp]
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 0))),
                Ok(Some(MaybeRelocatable::from(bigint!(imm))))
            );

            let code = r"ids.no_reg_ref";

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(
                py_result.map_err(|err| to_vm_error(err, py)),
                Err(to_vm_error(
                    PyValueError::new_err(VirtualMachineError::NoRegisterInReference.to_string()),
                    py
                ))
            );
        });
    }

    #[test]
    fn ids_reference_with_immediate() {
        Python::with_gil(|py| {
            let vm = PyVM::new(
                BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
                false,
                Vec::new(),
            );
            for _ in 0..2 {
                vm.vm.borrow_mut().add_memory_segment();
            }
            //Create references
            let mut references = HashMap::new();

            let imm_offset = 5;
            //Insert reference with inner_dereference and immediate value
            references.insert(
                String::from("inner_imm_ref"),
                HintReference {
                    register: Some(Register::FP),
                    offset1: 0,
                    offset2: 0,
                    dereference: false,
                    inner_dereference: true,
                    ap_tracking_data: None,
                    immediate: Some(bigint!(imm_offset)),
                    cairo_type: None,
                },
            );
            //Insert reference with dereference and immediate value
            references.insert(
                String::from("imm_ref"),
                HintReference {
                    register: Some(Register::FP),
                    offset1: 0,
                    offset2: 0,
                    dereference: true,
                    inner_dereference: false,
                    ap_tracking_data: None,
                    immediate: Some(bigint!(imm_offset)),
                    cairo_type: None,
                },
            );

            let relocatable = Relocatable::from((1, 3));
            vm.vm
                .borrow_mut()
                .insert_value(
                    &Relocatable::from((1, 0)),
                    &MaybeRelocatable::from(&relocatable),
                )
                .unwrap();

            let memory = PyMemory::new(&vm);

            let fp = PyRelocatable::from((1, 5));
            let ids = PyIds::new(
                &vm,
                &references,
                &ApTracking::default(),
                &HashMap::new(),
                Rc::new(HashMap::new()),
            );

            let globals = PyDict::new(py);
            globals
                .set_item("memory", PyCell::new(py, memory).unwrap())
                .unwrap();
            globals
                .set_item("fp", PyCell::new(py, fp).unwrap())
                .unwrap();
            globals
                .set_item("ids", PyCell::new(py, ids).unwrap())
                .unwrap();

            let code = r#"
assert ids.inner_imm_ref == ids.imm_ref
memory[fp] = ids.inner_imm_ref
"#;

            let py_result = py.run(code, Some(globals), None);

            assert_eq!(py_result.map_err(|err| to_vm_error(err, py)), Ok(()));
            //Check ids.inner_imm_ref is now at memory[fp]
            assert_eq!(
                vm.vm.borrow().get_maybe(&Relocatable::from((1, 5))),
                Ok(Some(MaybeRelocatable::from(relocatable + imm_offset)))
            );
        });
    }
}
