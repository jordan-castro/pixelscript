use std::ffi::c_void;

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
        // Null
        if vm.is_none(&obj) {
            return Ok(Var::new_null());
        }

        // Bool
        if obj.is_instance(vm.ctx.types.bool_type.into(), vm)? {
            let val = obj.try_to_bool(vm)?;
            return Ok(Var::new_bool(val));
        }

        // Int, might have to wrap this in a Var::Object
        if obj.is_instance(vm.ctx.types.int_type.into(), vm)? {
            let val = obj.try_to_value::<i64>(vm)?;
            return Ok(Var::new_i64(val));
        }

        // Float
        if obj.is_instance(vm.ctx.types.float_type.into(), vm)? {
            let pyref = obj.try_float(vm)?;
            let val = pyref.to_f64();

            return Ok(Var::new_f64(val));
        }

        // String
        if obj.is_instance(vm.ctx.types.str_type.into(), vm)? {
            let pyref = obj.str(vm)?;
            let val = pyref.as_str();

            return Ok(Var::new_string(val.to_string()));
        }

        // Generic Python object

        // Get the ptr to pyobject
        let ptr = PyObjectRef::into_raw(obj);

        Ok(Var::new_object(ptr as *mut c_void))
    }
}