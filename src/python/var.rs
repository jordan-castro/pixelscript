// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{ffi::c_void, sync::Arc};

use crate::{
    borrow_string, create_raw_string, free_raw_string, python::{
        func::py_assign,
        object::create_object,
        pocketpy::{self},
    }, shared::{
        object::get_object,
        var::{pxs_Var, pxs_VarType},
    }
};

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
    } else if tp == pocketpy::py_PredefinedType::tp_str as i32 {
        let cstr_ptr = unsafe { pocketpy::py_tostr(pref) };
        let r_str = borrow_string!(cstr_ptr).to_string();

        pxs_Var::new_string(r_str)
    } else if tp == pocketpy::py_PredefinedType::tp_NoneType as i32 || pref.is_null() {
        pxs_Var::new_null()
    } else if tp == pocketpy::py_PredefinedType::tp_list as i32 {
        // We have to get all items in the list
        let mut vars = vec![];
        let ok = unsafe { pocketpy::py_len(pref) };
        if !ok {
            return pxs_Var::new_null();
        }

        // Get list
        let list_len = unsafe { pocketpy::py_toint(pocketpy::py_retval()) };

        for i in 0..list_len {
            let item = unsafe { pocketpy::py_list_getitem(pref, i as i32) };
            if item.is_null() {
                continue;
            }

            vars.push(pocketpyref_to_var(item));
        }

        pxs_Var::new_list_with(vars)
    } else if tp == pocketpy::py_PredefinedType::tp_function as i32 {
        // Just like object, save the raw pointer
        pxs_Var::new_function(pref as *mut c_void, None)
    } else {
        pxs_Var::new_object(pref as *mut c_void, None)
    }
}

/// Convert a Var into a PocketPy ref
pub(super) fn var_to_pocketpyref(out: pocketpy::py_Ref, var: &pxs_Var) {
    unsafe {
        match var.tag {
            pxs_VarType::pxs_Int64 => {
                pocketpy::py_newint(out, var.get_i64().unwrap());
            }
            pxs_VarType::pxs_UInt64 => pocketpy::py_newint(out, var.get_u64().unwrap() as i64),
            pxs_VarType::pxs_Float64 => {
                pocketpy::py_newfloat(out, var.get_f64().unwrap());
            }
            crate::shared::var::pxs_VarType::pxs_Bool => {
                pocketpy::py_newbool(out, var.get_bool().unwrap());
            }
            crate::shared::var::pxs_VarType::pxs_String => {
                let s = var.get_string().unwrap();
                let c_str = create_raw_string!(s);
                pocketpy::py_newstr(out, c_str);
                // Free raw string
                free_raw_string!(c_str);
            }
            crate::shared::var::pxs_VarType::pxs_Null => {
                pocketpy::py_newnone(out);
            }
            crate::shared::var::pxs_VarType::pxs_Object => {
                if var.value.object_val.is_null() {
                    pocketpy::py_newnone(out);
                } else {
                    // This is a Python object that already exists, just that it's pointer was passed around.
                    let ptr = var.value.object_val as pocketpy::py_Ref;
                    py_assign(out, ptr);
                    // UNSAFE UNSAFE UNSAFE UNSAFE!!!!
                }
            }
            crate::shared::var::pxs_VarType::pxs_HostObject => {
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
                    let r0 = pocketpy::py_retval();
                    let module_name = pocketpy::py_tostr(r0);
                    let module_name = borrow_string!(module_name);
                    // Create the object for the first time...
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
                py_assign(out, *lang_ptr as pocketpy::py_Ref);
                // *out = *(*lang_ptr as pocketpy::py_Ref);
            }
            pxs_VarType::pxs_List => {
                // Ok take vars and convet them into pylist
                pocketpy::py_newlist(out);
                let list = var.get_list().unwrap();
                for i in 0..list.vars.len() {
                    let item = list.get_item(i as i32);
                    if let Some(item) = item {
                        // Add it
                        let tmp = pocketpy::py_pushtmp();
                        var_to_pocketpyref(tmp, item);
                        pocketpy::py_list_append(out, tmp);
                    }
                }

                // Donezo
            },
            pxs_VarType::pxs_Function => {
                if var.value.function_val.is_null() {
                    pocketpy::py_newnone(out);
                } else {
                    // Python function that already exists. Just a pointer passed around
                    let ptr = var.value.function_val as pocketpy::py_Ref;
                    py_assign(out, ptr);
                }

            },
        }
    }
}
