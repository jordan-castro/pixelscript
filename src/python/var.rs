// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{
    ffi::c_void,
    sync::Arc,
};

use crate::{
    borrow_string, create_raw_string, free_raw_string, pxs_debug, python::{
        consume_error, func::{get_from_obj, get_string_from_obj, py_assign}, object::create_object, pocketpy::{self}, python_pxs_get_register, python_pxs_new_register, python_pxs_remove_ref
    }, shared::{
        PtrMagic,
        object::get_object,
        pxs_Runtime,
        var::{pxs_Var, pxs_VarObject, pxs_VarType},
    }
};

/// Wrap a pointer with a Box!
/// 
/// This makes it possible to keep references to fun
pub(super) struct PythonPointer {
    /// Whether or not this is a index on the pocketpy stack
    pub is_index: bool,
    /// The raw pointer.
    pub ptr: *mut c_void
}
impl PtrMagic for PythonPointer {}
impl PythonPointer {
    fn with_index(index: i32) -> Self {
        PythonPointer { is_index: true, ptr: index as usize as *mut c_void }
    }

    // fn with_ref(py_ref: pocketpy::py_Ref) -> Self {
    //     PythonPointer { is_index: false, ptr: py_ref as *mut c_void }
    // }

    /// Get the Index pointer if `is_index` is `true`
    pub fn get_int(&self) -> i32 {
        if !self.is_index {
            -1
        } else {
            self.ptr as usize as i32 
        }
    }

    /// Get the `py_Ref` pointer.
    /// 
    /// Wrap this in a `PyStackGuard` if not null.
    pub fn get_ptr(&self) -> pocketpy::py_Ref {
        if self.is_index {
            let ok = python_pxs_get_register(self.get_int());
            if !ok {
                std::ptr::null_mut()
            } else {
                unsafe{
                    pocketpy::py_retval()
                }
            }
        } else {
            self.ptr as pocketpy::py_Ref
        }
    }
}

/// Create a internal PythonPointer. This uses integer references.
unsafe fn make_python_pointer(ptr: pocketpy::py_Ref) -> PythonPointer {
    // Get the object/function/list/set/tuple/dict/whatever as a IDX.
    let index = python_pxs_new_register(ptr);
    PythonPointer::with_index(index)
}

/// Frees a function and object memory.
/// This works via a stack IDX.
unsafe extern "C" fn free_py_mem(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let pp = PythonPointer::from_raw(ptr as *mut PythonPointer);
    // Remove from dict.
    python_pxs_remove_ref(pp.get_int());
}

/// Convert a PocketPy ref into a Var
pub(super) fn pocketpyref_to_var(pref: pocketpy::py_Ref) -> pxs_Var {
    let tp = unsafe { pocketpy::py_typeof(pref) } as i32;

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
    } else if tp == pocketpy::py_PredefinedType::tp_list as i32 || tp == pocketpy::py_PredefinedType::tp_tuple as i32 {
        // Have a guard
        let safe_ref = unsafe {pocketpy::py_pushtmp()};
        unsafe {py_assign(safe_ref, pref);}
        // Now go as normal.
        // We have to get all items in the list
        let mut vars = vec![];
        // Get len
        let ok = unsafe {pocketpy::py_len(safe_ref)};
        if !ok {
            return pxs_Var::new_exception(consume_error());
        }
        let len = unsafe{pocketpy::py_toint(pocketpy::py_retval())};

        // Parse through collection
        for i in 0..len {
            unsafe {
                // A tmp var for index
                let tmp = pocketpy::py_pushtmp();
                pocketpy::py_newint(tmp, i);

                let ok = pocketpy::py_getitem(safe_ref, tmp);
                if !ok {
                    return pxs_Var::new_exception(consume_error());
                }

                // We have a item!
                vars.push(pocketpyref_to_var(pocketpy::py_retval()));
                pocketpy::py_pop();
            }
        }
        // Pop list
        unsafe{ pocketpy::py_pop(); }
        
        pxs_Var::new_list_with(vars)
    } else if tp == pocketpy::py_PredefinedType::tp_function as i32 {
        pxs_Var::new_function(unsafe { make_python_pointer(pref).into_raw() as *mut c_void }, Some(free_py_mem))
    } else if tp == pocketpy::py_PredefinedType::tp_Exception as i32 {
        let msg = consume_error();
        pxs_Var::new_exception(msg)
    } 
    else {
        unsafe {
            // Check if object has `_pxs_ptr` assigned
            pxs_Var::new_object(pxs_VarObject::new_lang_only(make_python_pointer(pref).into_raw() as *mut c_void), Some(free_py_mem))
        }
    }
}

/// Convert a Var into a PocketPy ref
pub(super) fn var_to_pocketpyref(out: pocketpy::py_Ref, var: &pxs_Var, module_name: Option<&str>) {
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
                    let python_ptr = PythonPointer::from_borrow(var.get_object_ptr() as *mut PythonPointer);
                    let ptr = python_ptr.get_ptr();
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
                    // let module_name = get_module_name_from_obj_idx(id);
                    // Find current module
                    let obj_module_name = if let Some(module_name) = module_name {
                        module_name.to_string()
                    } else {
                        let cmod = pocketpy::py_inspect_currentmodule();
                        // Default to main if null.
                        if cmod.is_null() {
                            "__main__".to_string()
                        } else {
                            let name = get_string_from_obj(cmod, "__name__".to_string());
                            let pkg = get_string_from_obj(cmod, "__package__".to_string());
                            if pkg.len() > 0 {
                                format!("{pkg}.{name}")
                            } else {
                                name
                            }
                        }
                    };
                    // pxs_debug!("Full module path: {obj_module_name}");
                    // Create the object for the first time...
                    create_object(idx, Arc::clone(&pixel_object), &obj_module_name);
                    // Get py_retval
                    let pyobj = pocketpy::py_retval();
                    // Set that as the pointer
                    pixel_object.update_lang_ptr(pyobj as *mut c_void);
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
                        var_to_pocketpyref(tmp, item, module_name);
                        pocketpy::py_list_append(out, tmp);
                        pocketpy::py_pop();
                    }
                }

                // Donezo
            }
            pxs_VarType::pxs_Function => {
                if var.value.function_val.is_null() {
                    pocketpy::py_newnone(out);
                } else {
                    // Python function that already exists. Just a pointer passed around
                    let python_ptr = PythonPointer::from_borrow(var.value.function_val as *mut PythonPointer);
                    let ptr = python_ptr.get_ptr();
                    py_assign(out, ptr);
                }
            }
            pxs_VarType::pxs_Factory => {
                // Call and return
                let factory = var.get_factory().unwrap();
                let result = factory.call(pxs_Runtime::pxs_Python);
                // pxs_debug!("|FACTORY| result: {:#?}", result);
                // Convert to pocketpy
                var_to_pocketpyref(out, &result, module_name);
            }
            pxs_VarType::pxs_Exception => {
                // Raise exception
                pocketpy::py_newstr(out, var.value.string_val);
                let ok = pocketpy::py_tpcall(pocketpy::py_PredefinedType::tp_BaseException as i16, 1, out);
                if !ok {
                    let err = consume_error();
                    pxs_debug!("Exception could not be raised: {err}");
                    var_to_pocketpyref(out, &pxs_Var::new_exception(err), module_name);
                }

                py_assign(out, pocketpy::py_retval());
                pocketpy::py_raise(out);
            }
            pxs_VarType::pxs_Map => {
                // New dict
                pocketpy::py_newdict(out);
                let map = var.get_map().unwrap();
                let keys = map.keys();
                for k in keys {
                    let item = map.get_item(k);
                    if let Some(item) = item {
                        // Ok we can add this jaunt now
                        let py_key = pocketpy::py_pushtmp();
                        var_to_pocketpyref(py_key, k, module_name);
                        let py_value = pocketpy::py_pushtmp();
                        var_to_pocketpyref(py_value, item, module_name);

                        let ok = pocketpy::py_dict_setitem(out, py_key, py_value);
                        if !ok {
                            #[allow(unused)]
                            let err = consume_error();
                            pxs_debug!("Map to Python error: {err}");
                        }

                        // Pop stack (2)
                        pocketpy::py_pop();
                        pocketpy::py_pop();
                    }
                }

                // All good dayo!
            }
        }
    }
}
