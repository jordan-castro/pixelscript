// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{ffi::c_void, sync::Arc};

use crate::{borrow_string, create_raw_string, free_raw_string, python::{func::py_assign, object::create_object, pocketpy::{self, py_getreg}}, shared::{object::get_object, var::{pxs_Var, VarType}}};

/// Convert a PocketPy ref into a Var
pub(super) fn pocketpyref_to_var(pref: pocketpy::py_Ref) -> pxs_Var {
    let tp = unsafe { pocketpy::py_typeof(pref) } as i32;
    // let tp_enum = pocketpy::py_PredefinedType::from(tp);
    if tp == pocketpy::py_PredefinedType::tp_int as i32 {
        let val = unsafe { pocketpy::py_toint(pref) };
        pxs_Var::new_i64(val)
    } else if tp == pocketpy::py_PredefinedType::tp_float as i32 {
        let val = unsafe { pocketpy::py_tofloat(pref) };
        pxs_Var::new_f64(val)
    } else if tp == pocketpy::py_PredefinedType::tp_bool as i32 {
        let val = unsafe { pocketpy::py_tobool(pref) };
        pxs_Var::new_bool(val)
    }  else if tp == pocketpy::py_PredefinedType::tp_str as i32 {
        let cstr_ptr = unsafe { pocketpy::py_tostr(pref) };
        let r_str = borrow_string!(cstr_ptr).to_string();

        pxs_Var::new_string(r_str)
    } else if tp == pocketpy::py_PredefinedType::tp_NoneType as i32 {
        pxs_Var::new_null()
    }
    else {
        pxs_Var::new_object(pref as *mut c_void)
    }
}

/// Convert a Var into a PocketPy ref
pub(super) fn var_to_pocketpyref(out: pocketpy::py_Ref, var: &pxs_Var) {
    unsafe {
        match var.tag {
            VarType::Int64 => {
                pocketpy::py_newint(out, var.get_i64().unwrap());
            },
            VarType::UInt64 => {
                pocketpy::py_newint(out, var.get_u64().unwrap() as i64)
            },
            VarType::Float64 => {
                pocketpy::py_newfloat(out, var.get_f64().unwrap());
            },
            crate::shared::var::VarType::Bool => {
                pocketpy::py_newbool(out, var.get_bool().unwrap());
            },
            crate::shared::var::VarType::String => {
                let s = var.get_string().unwrap();
                let c_str = create_raw_string!(s);
                pocketpy::py_newstr(out, c_str);
                // Free raw string
                free_raw_string!(c_str);
            },
            crate::shared::var::VarType::Null => {
                pocketpy::py_newnone(out);
            },
            crate::shared::var::VarType::Object => {
                if var.value.object_val.is_null() {
                    pocketpy::py_newnone(out);
                } else {
                    // This is a Python object that already exists, just that it's pointer was passed around.
                    let ptr = var.value.object_val as pocketpy::py_Ref;
                    println!("ptr Type: {:#?}", pocketpy::py_typeof(ptr));
                    py_assign(out, ptr);
                    println!("out Type: {:#?}", pocketpy::py_typeof(out));
                    // UNSAFE UNSAFE UNSAFE UNSAFE!!!!
                }
            },
            crate::shared::var::VarType::HostObject => {
                let idx = var.value.host_object_val;
                let pixel_object = get_object(idx).unwrap();
                // DO NOT FREE POCKETPY memory.
                pixel_object.update_free_lang_ptr(false);
                let lang_ptr_is_null = pixel_object.lang_ptr.lock().unwrap().is_null();
                if lang_ptr_is_null {
                    // Find current module
                    let cmod = pocketpy::py_inspect_currentmodule();
                    let c_name = create_raw_string!("__name__");
                    let pyname = pocketpy::py_name(c_name);
                    pocketpy::py_getattr(cmod, pyname);
                    let r0 = py_getreg(0);
                    let module_name = pocketpy::py_tostr(r0);
                    let module_name = borrow_string!(module_name);
                    // TODO: Create the object for the first time...
                    create_object(idx, Arc::clone(&pixel_object), module_name);
                    // Get py_retval
                    let pyobj = pocketpy::py_retval();
                    // Set that as the pointer
                    pixel_object.update_lang_ptr(pyobj as *mut c_void);
                    free_raw_string!(c_name);
                }
                // Get PTR again
                let lang_ptr = pixel_object.lang_ptr.lock().unwrap();
                // Assign again
                *out = *(*lang_ptr as pocketpy::py_Ref);
            },
        }
    }
}

// RUST PYTHON OLD VERSION
            // crate::shared::var::VarType::HostObject => {
            //     unsafe {
            //         let idx = var.value.host_object_val;
            //         let pixel_object = get_object(idx).unwrap();
            //         let lang_ptr_is_null = pixel_object.lang_ptr.lock().unwrap().is_null();
            //         if lang_ptr_is_null {
            //             // Create the object for the first and mutate the pixel object TODO.
            //             let pyobj = create_object(vm, idx, Arc::clone(&pixel_object));
            //             // Set pointer
            //             pixel_object.update_lang_ptr(pyobj.into_raw() as *mut c_void);
            //         }

            //         // Get PTR again
            //         let lang_ptr = pixel_object.lang_ptr.lock().unwrap();
            //         // Get as PyObject and grab dict
            //         let pyobj_ptr = *lang_ptr as *const PyObject;

            //         PyObjectRef::from_raw(pyobj_ptr)
            //     }
            // },
