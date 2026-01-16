// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{cell::RefCell, collections::{HashMap, HashSet}};

use parking_lot::{ReentrantMutex, ReentrantMutexGuard};

use crate::{
    create_raw_string, free_raw_string, own_string,
    python::{func::{pocketpy_bridge, virtual_module_loader}, module::create_module, var::{pocketpyref_to_var, var_to_pocketpyref}},
    shared::{PixelScript, var::{ObjectMethods, Var}},
};

// Allow for the binidngs only
#[allow(unused)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
pub(self) mod pocketpy {
    include!(concat!(env!("OUT_DIR"), "/pocketpy_bindings.rs"));
}

mod func;
mod var;
mod object;
mod module;

thread_local! {
    static PYSTATE: ReentrantMutex<State> = ReentrantMutex::new(init_state());
}

/// This is the Pocketpy state. Each language gets it's own private state
struct State {
    /// Name to IDX lookup for pocketpy bridge
    name_to_idx: RefCell<HashMap<i32, HashMap<String, i32>>>,

    /// Keep a list of defined PixelObject as class
    defined_objects: RefCell<HashMap<i32, HashSet<String>>>,
    
    /// Current thread idx
    thread_idx: RefCell<i32>
}

pub(self) fn exec_py(code: &str, name: &str, module: &str) -> String {
    run_py(code, name, pocketpy::py_CompileMode_EXEC_MODE, Some(module))
}

fn run_py(code: &str, name: &str, comp_mode: pocketpy::py_CompileMode, module: Option<&str>) -> String {
    let c_code = create_raw_string!(code);
    let c_name = create_raw_string!(name);
    unsafe {
        let res = {
        if let Some(module_name) = module {
            let c_module = create_raw_string!(module_name);
            let pymod = pocketpy::py_getmodule(c_module);
            free_raw_string!(c_module);

            pocketpy::py_exec(c_code, c_name, comp_mode, pymod)
        } else {
            pocketpy::py_exec(c_code, c_name, comp_mode, std::ptr::null_mut())
        }
    };
        free_raw_string!(c_code);
        free_raw_string!(c_name);
        if !res {
            let py_res = pocketpy::py_formatexc();
            let py_res = own_string!(py_res);

            py_res
        } else {
            String::new()
        }
    }
}

pub(self) fn eval_main_py(code: &str, name: &str) -> String {
    run_py(code, name, pocketpy::py_CompileMode_EVAL_MODE, None)
}

pub(self) fn eval_py(code: &str, name: &str, module_name: &str) -> String {
    run_py(code, name, pocketpy::py_CompileMode_EVAL_MODE, Some(module_name))
}

pub(self) fn exec_main_py(code: &str, name: &str) -> String {
    run_py(code, name, pocketpy::py_CompileMode_EXEC_MODE, None)
    // let c_code = create_raw_string!(code);
    // let c_name = create_raw_string!(name);
    // unsafe {
    //     let res = pocketpy::py_exec(c_code, c_name, pocketpy::py_CompileMode_EXEC_MODE, std::ptr::null_mut());
    //     free_raw_string!(c_code);
    //     free_raw_string!(c_name);
    //     if !res {
    //         let py_res = pocketpy::py_formatexc();
    //         let py_res = own_string!(py_res);

    //         py_res
    //     } else {
    //         String::new()
    //     }
    // }
}

/// Initialize Lua state per thread.
fn init_state() -> State {
    State {
        name_to_idx: RefCell::new(HashMap::new()),
        defined_objects: RefCell::new(HashMap::new()),
        thread_idx: RefCell::new(0)
    }
}

/// Get the state of Pocketpy.
pub(self) fn get_py_state() -> ReentrantMutexGuard<'static, State> {
    PYSTATE.with(|mutex| {
        let guard = mutex.lock();
        // Transmute the lifetime so the guard can be passed around the thread
        unsafe { std::mem::transmute(guard) }
    })
}

/// Add a new name => idx
pub(self) fn add_new_name_idx_fn(name: String, idx: i32) {
    let state = get_py_state();
    let t = state.thread_idx.borrow();
    let mut names = state.name_to_idx.borrow_mut();
    if let Some(h) = names.get_mut(&t) {
        h.insert(name, idx);
    } else {
        let mut map = HashMap::new();
        map.insert(name, idx);
        names.insert(t.clone(), map);
    }
}

/// Get a IDX from a name
pub(self) fn get_fn_idx_from_name(name: &str) -> Option<i32> {
    let state = get_py_state();
    let t = state.thread_idx.borrow();
    if let Some(m) = state.name_to_idx.borrow().get(&t){
        m.get(name).cloned()
    } else {
        None
    }
}

/// Add a new defined object
pub(self) fn add_new_defined_object(name: &str) {
    let state = get_py_state();
    let t = state.thread_idx.borrow();
    let mut names = {
        if let Some(names) = state.defined_objects.borrow_mut().get(&t).cloned() {
            names
        } else {
            let set = HashSet::new();
            set
        }
    };

    names.insert(name.to_string());
} 

/// Check if a object is already defined
pub(self) fn is_object_defined(name: &str) -> bool {
    let state = get_py_state();
    let t = state.thread_idx.borrow();
    if let Some(set) = state.defined_objects.borrow().get(&t) {
        set.contains(name)
    } else {
        false
    }
} 

pub(self) fn make_private(name: &str) -> String {
    format!("_pxs_{}", name)
}

/// This needs to be called in every PKPY VM.
unsafe fn setup_module_loader() {
    unsafe {
        //             pocketpy::py_bindfunc(global_scope, c_name, Some(pocketpy_bridge));
        let main_name = create_raw_string!("__main__");
        let func_name = create_raw_string!("__import__");
        let main = pocketpy::py_getmodule(main_name);
        pocketpy::py_bindfunc(main, func_name, Some(virtual_module_loader));

        free_raw_string!(main_name);
        free_raw_string!(func_name);
    }
}

pub struct PythonScripting;

impl PixelScript for PythonScripting {
    fn start() {
        // py initialize here
        // let pxs_globals: pocketpy::py_Ref;
        unsafe {
            pocketpy::py_initialize();
            // Create _pxs_globals
            // let pxs_name = create_raw_string!("_pxs_globals");
            // pxs_globals = pocketpy::py_newmodule(pxs_name);
            setup_module_loader();
        }
        // let _s = exec_main_py("1 + 1", "<init>");
        let _state = get_py_state();
    }

    fn stop() {
        unsafe {
            pocketpy::py_finalize();
        }
    }

//     fn add_variable(name: &str, variable: &crate::shared::var::Var) {
//         unsafe {
//             let r0 = pocketpy::py_getreg(0);
//             if r0.is_null() {
//                 return;
//             }
//             let cstr = create_raw_string!(name);
//             let pyname = pocketpy::py_name(cstr);
//             var_to_pocketpyref(r0, variable);
//             pocketpy::py_setglobal(pyname, r0);
//             // free cstr
//             free_raw_string!(cstr);
//         }
//     }

//     fn add_callback(name: &str, idx: i32) {
//         // Save function
//         add_new_name_idx_fn(name.to_string(), idx);

//         // Create a "private" name
//         let private_name = make_private(name);

//         let c_name = create_raw_string!(private_name.clone());
//         let c_main = create_raw_string!("__main__");
//         let bridge_code = format!(
//             r#"
// def _{name}_(*args):
//     return {private_name}('{name}', *args)
// "#
//         );
//         let c_brige_name = format!("<callback_bridge for {private_name}>");
//         unsafe {
//             let global_scope = pocketpy::py_getmodule(c_main);

//             pocketpy::py_bindfunc(global_scope, c_name, Some(pocketpy_bridge));

//             // Execute bridge
//             let s = exec_main_py(&bridge_code, &c_brige_name);
//             free_raw_string!(c_name);
//             free_raw_string!(c_main);
//         }
//     }

    fn add_module(source: std::sync::Arc<crate::shared::module::Module>) {
        create_module(&source, None);
    }

    fn execute(code: &str, file_name: &str) -> String {
        let res = exec_main_py(code, file_name);
        res
    }
    
    fn start_thread() {
        unsafe {
            let idx = pocketpy::py_currentvm() + 1;
            pocketpy::py_switchvm(idx);
            setup_module_loader();
            let state = get_py_state();
            *(state.thread_idx.borrow_mut()) = idx;
        }
    }
    
    fn stop_thread() {
        unsafe {
            let idx = pocketpy::py_currentvm() - 1;
            pocketpy::py_resetvm();
            pocketpy::py_switchvm(idx);
            let state = get_py_state();
            *(state.thread_idx.borrow_mut()) = idx;
        }
    }
}

impl ObjectMethods for PythonScripting {
    fn object_call(var: &crate::shared::var::Var, method: &str, args: &Vec<crate::shared::var::Var>) -> Result<crate::shared::var::Var, anyhow::Error> {
        // Get the pyref
        let pyref = unsafe { pocketpy::py_getreg(0) };
        var_to_pocketpyref(pyref, var);

        let method_name = create_raw_string!(method);
        // Call a method on it.
        unsafe {
            let pymethod_name = pocketpy::py_name(method_name);
            pocketpy::py_getattr(pyref, pymethod_name);
            // Get the result pushed to the stack.
            let pymethod = pocketpy::py_getreg(0);

            // Convert args into py_Ref
            for i in 0..args.len() {
                let pyref = pocketpy::py_getreg((i + 1) as i32);
                var_to_pocketpyref(pyref, &args[i]);
                pocketpy::py_push(pyref);
            }

            // Now call
            pocketpy::py_call(pymethod, args.len() as i32, std::ptr::null_mut());
        }

        // Result is py_retval
        let result = unsafe { pocketpy::py_retval() };
        Ok(pocketpyref_to_var(result))
    }
}
