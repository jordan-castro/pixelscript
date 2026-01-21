// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::{
    borrow_string, create_raw_string, free_raw_string,
    python::{
        get_fn_idx_from_name, pocketpy, var::pocketpyref_to_var, var_to_pocketpyref,
    },
    shared::{PixelScriptRuntime, func::call_function, var::pxs_Var},
};

/// The size of the pyType thingy. This is the same for all modes and platforms.
/// According to bluelove
const PY_TYPE_SIZE: usize = 24;

/// Use instead of the py_arg macro.
pub(super) unsafe fn py_get_arg(argv: pocketpy::py_StackRef, i: usize) -> pocketpy::py_StackRef {
    let base_addr = argv as *mut u8;
    let offset_addr = unsafe { base_addr.add(i * PY_TYPE_SIZE) };
    offset_addr as pocketpy::py_StackRef
}

/// Use instead of the py_assign macro.
pub(super) unsafe fn py_assign(left: pocketpy::py_Ref, right: pocketpy::py_Ref) {
    unsafe {
        std::ptr::copy_nonoverlapping(right, left, 1);
    }
}

// /// Use instead of py_setattr
// pub(super) unsafe fn py_setattr(_self: pocketpy::py_Ref, name: &str, val: pocketpy::py_Ref) {
//     let name = create_raw_string!(name);
//     unsafe {
//         let pyname = pocketpy::py_name(name);
//         pocketpy::py_setattr(_self, pyname, val);
//         free_raw_string!(name);
//     }
// }

pub(super) unsafe fn raise(msg: &str) -> bool {
    let c_msg = create_raw_string!(msg);
    unsafe {
        let ret_slot = pocketpy::py_retval();
        pocketpy::py_newstr(ret_slot, c_msg);
        free_raw_string!(c_msg);

        return true;
    }
}

/// The pocketpy bridge
pub(super) unsafe extern "C" fn pocketpy_bridge(argc: i32, argv: pocketpy::py_StackRef) -> bool {
    // let pyref_size = pocketpy::get_py_TValue_size();
    if argc < 1 {
        unsafe {
            return raise("Python: argc < 1");
        }
        // var_to_pocketpyref(ret_slot, &Var::new_null());
    }
    let c_name = unsafe { pocketpy::py_tostr(py_get_arg(argv, 0)) };
    let name = borrow_string!(c_name);
    let fn_idx = get_fn_idx_from_name(name);
    if fn_idx.is_none() {
        return unsafe { raise("Python: fn_idx is empty.") };
    }
    let fn_idx = fn_idx.unwrap();

    // Convert argv into Vec<Var>
    let mut vars: Vec<pxs_Var> = vec![];

    // Add the runtime
    vars.push(pxs_Var::new_i64(PixelScriptRuntime::Python as i64));

    // Convert py_Ref into pxs_Var.
    for i in 1..argc {
        let arg_ref = unsafe { py_get_arg(argv, i as usize) };
        vars.push(pocketpyref_to_var(arg_ref));
    }

    // Call internal function
    unsafe {
        let res = call_function(fn_idx, vars);
        let ret_slot = pocketpy::py_retval();

        var_to_pocketpyref(ret_slot, &res);
    }
    true
}
