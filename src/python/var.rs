use std::any::Any;

use anyhow::{Error, anyhow};
use rustpython::vm::{PyObject, PyObjectRef, TryFromObject, convert::ToPyObject};

use crate::shared::{object::get_object_lookup, var::Var};

impl ToPyObject for Var {
    fn to_pyobject(self, vm: &rustpython::vm::VirtualMachine) -> rustpython::vm::PyObjectRef {
        match self.tag {
            crate::shared::var::VarType::Int32 => {
                vm.ctx.new_int(self.get_i32().unwrap()).into()
            },
            crate::shared::var::VarType::Int64 => {
                vm.ctx.new_int(self.get_i64().unwrap()).into()
            },
            crate::shared::var::VarType::UInt32 => {
                vm.ctx.new_int(self.get_u32().unwrap()).into()
            },
            crate::shared::var::VarType::UInt64 => {
                vm.ctx.new_int(self.get_u64().unwrap()).into()
            },
            crate::shared::var::VarType::String => {
                let contents = self.get_string().unwrap();
                vm.ctx.new_str(contents).into()
            },
            crate::shared::var::VarType::Bool => {
                vm.ctx.new_bool(self.get_bool().unwrap()).into()
            },
            crate::shared::var::VarType::Float32 => {
                vm.ctx.new_float(self.get_bigfloat()).into()
            },
            crate::shared::var::VarType::Float64 => {
                vm.ctx.new_float(self.get_bigfloat()).into()
            },
            crate::shared::var::VarType::Null => vm.ctx.none(),
            crate::shared::var::VarType::Object => {
                unsafe {
                    // This is a Python Class
                    let pyobj_ptr = self.value.object_val as *const PyObject;

                    PyObjectRef::from_raw(pyobj_ptr)
                }
            },
            crate::shared::var::VarType::HostObject => {
                unsafe {
                    let idx = self.value.host_object_val;
                    let object_lookup = get_object_lookup();
                    let pixel_object = object_lookup.get_object(idx).unwrap().clone();
                    let lang_ptr_is_null = pixel_object.lang_ptr.lock().unwrap().is_null();
                    if lang_ptr_is_null {
                        // Create the object for the first and mutate the pixel object TODO.
                    }

                    // Get PTR again
                    let lang_ptr = pixel_object.lang_ptr.lock().unwrap();
                    // Get as PyObject and grab dict
                    let pyobj_ptr = *lang_ptr as *const PyObject;

                    PyObjectRef::from_raw(pyobj_ptr)
                }
            },
        }
    }
}

impl TryFromObject for Var {
    fn try_from_object(vm: &rustpython::vm::VirtualMachine, obj: rustpython::vm::PyObjectRef) -> rustpython::vm::PyResult<Self> {
        if vm.is_none(&obj) {
            return Ok(Var::new_null());
        }

        if obj.is_instance(vm.ctx.types.bool_type.into(), vm)? {
            let val = obj.try_to_bool(vm)?;
            return Ok(Var::new_bool(val));
        }

        if obj.is_instance(cls, vm)

        return Error("");
    }
}