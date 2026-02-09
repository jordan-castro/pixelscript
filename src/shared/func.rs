// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::shared::{
    PtrMagic,
    var::{default_deleter, pxs_VarList, pxs_VarType, pxs_VarValue},
};

use super::var::pxs_Var;
use std::{
    cell::Cell, collections::HashMap, ffi::c_void, sync::{Mutex, OnceLock}
};

/// Function reference used in C.
///
/// args: *mut pxs_Var, A list of vars.
/// opaque: *mut c_void, opaque user data.
///
/// Func handles it's own memory, so no need to free the *mut Var returned or the argvs.
///
/// But if you use any Vars within the function, you will have to free them before the function returns.
#[allow(non_camel_case_types)]
pub type pxs_Func = unsafe extern "C" fn(args: *mut pxs_Var, opaque: *mut c_void) -> *mut pxs_Var;

/// Basic rust structure to track Funcs and opaques together.
pub struct Function {
    pub name: String,
    pub func: pxs_Func,
    pub opaque: *mut c_void,
}

unsafe impl Send for Function {}
unsafe impl Sync for Function {}

/// Lookup state structure
pub struct FunctionLookup {
    /// Function hash shared between all runtimes.
    ///
    /// Negative numbers are valid here.
    pub function_hash: HashMap<i32, Function>,
}

impl FunctionLookup {
    pub fn get_function(&self, idx: i32) -> Option<&Function> {
        self.function_hash.get(&idx)
    }
    pub fn add_function(&mut self, name: &str, func: pxs_Func, opaque: *mut c_void) -> i32 {
        // TODO: Allow for negative idxs.
        self.function_hash.insert(
            self.function_hash.len() as i32,
            Function {
                name: name.to_string(),
                func,
                opaque,
            },
        );

        return (self.function_hash.len() - 1) as i32;
    }
}

/// The function lookup!
static FUNCTION_LOOKUP: OnceLock<Mutex<FunctionLookup>> = OnceLock::new();

/// Get the function lookup global state. Shared between all runtimes.
fn get_function_lookup() -> std::sync::MutexGuard<'static, FunctionLookup> {
    FUNCTION_LOOKUP
        .get_or_init(|| {
            Mutex::new(FunctionLookup {
                function_hash: HashMap::new(),
            })
        })
        .lock()
        .unwrap()
}

/// Add a function to the lookup
pub fn lookup_add_function(name: &str, func: pxs_Func, opaque: *mut c_void) -> i32 {
    let mut lookup = get_function_lookup();
    let idx = lookup.function_hash.len();
    lookup.function_hash.insert(
        idx as i32,
        Function {
            name: name.to_string(),
            func,
            opaque,
        },
    );
    idx as i32
}

/// Clear function lookup hash
pub fn clear_function_lookup() {
    let mut lookup = get_function_lookup();
    lookup.function_hash.clear();
}

/// Call a function that is saved in the lookup by a idx.
///
/// This should only be used within languages and never from a end user.
pub unsafe fn call_function(fn_idx: i32, args: Vec<pxs_Var>) -> pxs_Var {
    let (func, opaque) = {
        let fl = get_function_lookup();
        let function = fl.get_function(fn_idx);
        if function.is_none() {
            return pxs_Var::new_null();
        }

        let function = function.unwrap();

        (function.func, function.opaque)
    };

    // Convert the pxs_Var vector into a list.
    // Do this because I don't want to mess with the older code.
    let args = pxs_Var {
        tag: pxs_VarType::pxs_List,
        value: pxs_VarValue {
            list_val: pxs_VarList { vars: args }.into_raw(),
        },
        deleter: Cell::new(default_deleter)
    };
    let args_ptr = args.into_raw();

    unsafe {
        let res = func(args_ptr, opaque);
        // Free args
        let _ = pxs_Var::from_raw(args_ptr);

        if res.is_null() {
            pxs_Var::new_null()
        } else {
            pxs_Var::from_raw(res)
        }
    }
}
