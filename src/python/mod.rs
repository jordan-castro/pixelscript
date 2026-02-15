// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use anyhow::anyhow;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};

use crate::{
    borrow_string, create_raw_string, free_raw_string, own_string,
    python::{
        func::pocketpy_bridge,
        module::create_module,
        var::{pocketpyref_to_var, var_to_pocketpyref},
    },
    shared::{PixelScript, read_file, read_file_dir, var::{ObjectMethods, pxs_Var, pxs_VarList}},
};

// Allow for the binidngs only
#[allow(unused)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub(self) mod pocketpy {
    include!(concat!(env!("OUT_DIR"), "/pocketpy_bindings.rs"));
}

mod func;
mod module;
mod object;
mod var;

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
    thread_idx: RefCell<i32>,
}

pub(self) fn exec_py(code: &str, name: &str, module: &str) -> String {
    run_py(
        code,
        name,
        pocketpy::py_CompileMode::EXEC_MODE,
        Some(module),
    )
}

fn run_py(
    code: &str,
    name: &str,
    comp_mode: pocketpy::py_CompileMode,
    module: Option<&str>,
) -> String {
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

            // Clear the exception
            pocketpy::py_clearexc(std::ptr::null_mut());

            py_res
        } else {
            String::new()
        }
    }
}

#[allow(unused)]
pub(self) fn eval_main_py(code: &str, name: &str) -> String {
    run_py(code, name, pocketpy::py_CompileMode::EVAL_MODE, None)
}

pub(self) fn eval_py(code: &str, name: &str, module_name: &str) -> String {
    run_py(
        code,
        name,
        pocketpy::py_CompileMode::EVAL_MODE,
        Some(module_name),
    )
}

pub(self) fn exec_main_py(code: &str, name: &str) -> String {
    run_py(code, name, pocketpy::py_CompileMode::EXEC_MODE, None)
}

/// Initialize Lua state per thread.
fn init_state() -> State {
    State {
        name_to_idx: RefCell::new(HashMap::new()),
        defined_objects: RefCell::new(HashMap::new()),
        thread_idx: RefCell::new(0),
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
    if let Some(m) = state.name_to_idx.borrow().get(&t) {
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

pub(self) fn make_private_prefix(name: &str, prefix: &str) -> String {
    make_private(format!("{prefix}_{name}").as_str())
}

/// This is the import overrider
unsafe extern "C" fn import_file(arg1: *const std::ffi::c_char) -> *mut std::ffi::c_char {
    // Borrow string
    let b = borrow_string!(arg1);
    // Remove .py and check if this is a directory
    let file_path = {
        let pos_dir = &b[0..b.len() - 3];
        let files = read_file_dir(pos_dir);

        if files.contains(&"__import__.py".to_string()) {
            // Ok just use that then
            format!("{pos_dir}__import__.py")
        } else {
            // No __import__.py so let's see first if there is any .py so we can return a pseudo type
            for _ in files.iter() {
                return create_raw_string!("");
            }
            b.to_string()
        }
    };

    let contents = read_file(&file_path);

    if contents.is_empty() {
        std::ptr::null_mut()
    } else {
        create_raw_string!(contents)
    }
}

/// This needs to be called in every PKPY VM.
unsafe fn setup_module_loader() {
    unsafe {
        let callbacks = pocketpy::py_callbacks();
        (*callbacks).importfile = Some(import_file);
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

    fn add_module(source: std::sync::Arc<crate::shared::module::pxs_Module>) {
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
            // clear current
            Self::clear_state(false);

            let idx = pocketpy::py_currentvm() - 1;
            pocketpy::py_resetvm();
            pocketpy::py_switchvm(idx);
            let state = get_py_state();
            *(state.thread_idx.borrow_mut()) = idx;
        }
    }
    
    fn clear_state(call_gc: bool) {
        // Drop defined objects
        let state = get_py_state();
        state.defined_objects.borrow_mut().clear();
        let idx = state.thread_idx.borrow().abs();
        let mut binding = state.name_to_idx.borrow_mut();
        let name_map = binding.get_mut(&idx);
        if let Some(m) = name_map {
            m.clear();
        }

        if call_gc {
            // Invoke GC
            unsafe {
                pocketpy::py_gc_collect();
            }
        }
    }
    
    fn eval(code: &str) -> pxs_Var {
        let res = exec_main_py(code, "eval");
        if res.is_empty() {
            pocketpyref_to_var(unsafe{ pocketpy::py_retval() })
        } else {
            pxs_Var::new_string(res)
        }
    }

    
}

/// Add pxs vars to the stack
fn add_args(args: &Vec<pxs_Var>) {
    // Convert args into py_Ref
    for i in 0..args.len() {
        let pyref = unsafe { pocketpy::py_pushtmp() };
        var_to_pocketpyref(pyref, &args[i]);
    }
}

impl ObjectMethods for PythonScripting {
    fn object_call(
        var: &crate::shared::var::pxs_Var,
        method: &str,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        // Make a object ref
        let obj_ref = unsafe { pocketpy::py_pushtmp() };
        // Set it
        var_to_pocketpyref(obj_ref, var);

        let method_name = create_raw_string!(method);
        // Call a method on it.
        unsafe {
            let pymethod_name = pocketpy::py_name(method_name);
            pocketpy::py_getattr(obj_ref, pymethod_name);
            // Get the result pushed to the stack.
            let pymethod = pocketpy::py_retval();
            free_raw_string!(method_name);

            // Push method
            pocketpy::py_push(pymethod);
            // Push self
            pocketpy::py_push(obj_ref);

            add_args(&args.vars);

            // Now call
            // Result is py_retval
            // Call it via vectrocall
            let ok = pocketpy::py_vectorcall(args.vars.len() as u16, 0);
            if !ok {
                return Ok(pxs_Var::new_null());
            }

            let result_ref = pocketpy::py_retval();
            let final_var = pocketpyref_to_var(result_ref);
            
            Ok(final_var)
        }

    }

    fn call_method(
        method: &str,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        // Convert methods to pocketpy
        let method_name = create_raw_string!(method);
        unsafe {
            let pymethod_name = pocketpy::py_name(method_name);
            let pymethod = {
                // Try a builtin first
                let global = pocketpy::py_getbuiltin(pymethod_name);
                if !global.is_null() {
                    global
                } else {
                    // Then look for a method in current module
                    let found = pocketpy::py_getattr(pocketpy::py_inspect_currentmodule(), pymethod_name);
                    if !found {
                        std::ptr::null_mut()
                    } else {
                        pocketpy::py_retval()
                    }
                }
            };
            free_raw_string!(method_name);
            
            if pymethod.is_null() {
                return Ok(pxs_Var::new_null());
            }

            // Push method
            pocketpy::py_push(pymethod);
            // Push self, in this case nil.
            pocketpy::py_pushnil();

            add_args(&args.vars);

            // Call it via vectrocall
            let ok = pocketpy::py_vectorcall(args.vars.len() as u16, 0);
            if !ok {
                return Ok(pxs_Var::new_null());
            }

            let result_ref = pocketpy::py_retval();
            let final_var = pocketpyref_to_var(result_ref);
            
            Ok(final_var)
        }
    }
    
    fn var_call(method: &pxs_Var, args: &mut pxs_VarList) -> Result<pxs_Var, anyhow::Error> {
        // Make sure it's a function!
        if !method.is_function() {
            return Err(anyhow!("Expected Function, found: {:#?}", method.tag));
        }

        // Get ptr as py_ref
        let fn_ptr = method.get_function().unwrap();
        let pyfn = fn_ptr as pocketpy::py_Ref;

        // Now prepare the stack!
        unsafe {
            pocketpy::py_push(pyfn);
            pocketpy::py_pushnil();
        }

        // Add args
        add_args(&args.vars);

        // Call it via vectrocall
        let ok = unsafe { pocketpy::py_vectorcall(args.vars.len() as u16, 0) };
        if !ok {
            return Ok(pxs_Var::new_null());
        }

        unsafe { Ok(pocketpyref_to_var(pocketpy::py_retval())) }
    }
    
    fn get(var: &pxs_Var, key: &str) -> Result<pxs_Var, anyhow::Error> {
        unsafe {
            if var.value.object_val.is_null() {
                return Err(anyhow!("var.value.object_val is Null"));
            }
            // Deref
            let object = var.value.object_val as pocketpy::py_Ref;
            let raw_key = create_raw_string!(key);
            let py_key = pocketpy::py_name(raw_key);
            free_raw_string!(raw_key);
            let res = pocketpy::py_getattr(object, py_key);

            if !res {
                return Err(anyhow!("var.{} could not be done.", key));
            }

            // Get value
            Ok(
                pocketpyref_to_var(pocketpy::py_retval())
            )
        }
    }
    
    fn set(var: &pxs_Var, key: &str, value: &pxs_Var) -> Result<pxs_Var, anyhow::Error> {
        unsafe {
            if var.value.object_val.is_null() {
                return Err(anyhow!("var.value.object_val is Null"));
            }

            // Deref
            let object = var.value.object_val as pocketpy::py_Ref;
            // Key
            let raw_key = create_raw_string!(key);
            let py_key = pocketpy::py_name(raw_key);
            free_raw_string!(raw_key);
            // Set
            let tmp = pocketpy::py_pushtmp();
            var_to_pocketpyref(tmp, value);
            let res = pocketpy::py_setattr(object, py_key, tmp);

            if !res {
                return Err(anyhow!("var.{} = {:#?} could not be done.", key, value))
            }

            Ok(
                pocketpyref_to_var(pocketpy::py_retval())
            )
        }
    }

    
}
