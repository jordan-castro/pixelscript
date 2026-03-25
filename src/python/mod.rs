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

use anyhow::{Result, anyhow};
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};

use crate::{
    borrow_string, create_raw_string, free_raw_string, own_string, pxs_debug,
    python::{
        func::{get_builtin, pocketpy_bridge, py_assign},
        module::create_module,
        var::{PythonPointer, pocketpyref_to_var, var_to_pocketpyref},
    },
    shared::{
        PixelScript, PtrMagic, read_file, read_file_dir, var::{ObjectMethods, pxs_Var, pxs_VarList}
    },
    with_feature,
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

/// Execute python code on a certain module.
pub(self) fn exec_py(code: &str, name: &str, module: &str) -> String {
    run_py(
        code,
        name,
        pocketpy::py_CompileMode::EXEC_MODE,
        Some(module),
    )
}

/// Run python code as eval or exec on a optinal module.
/// If no module is chosen, it defaults to __main__ via pocketpy internals.
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
        if !res { consume_error() } else { String::new() }
    }
}

/// Concumes the current py-error/exception and returns it as a string.
pub(self) fn consume_error() -> String {
    unsafe {
        let err = pocketpy::py_formatexc();
        if err.is_null() {
            return String::new();
        }

        let res = own_string!(err);

        // Clear
        pocketpy::py_clearexc(std::ptr::null_mut());
        res
    }
}

/// Evaluate python code in the main module.
#[allow(unused)]
pub(self) fn eval_main_py(code: &str, name: &str) -> String {
    run_py(code, name, pocketpy::py_CompileMode::EVAL_MODE, None)
}

/// Evaluate python code in a certain module.
pub(self) fn eval_py(code: &str, name: &str, module_name: &str) -> String {
    run_py(
        code,
        name,
        pocketpy::py_CompileMode::EVAL_MODE,
        Some(module_name),
    )
}

/// Execute python code in the main module.
pub(self) fn exec_main_py(code: &str, name: &str) -> String {
    run_py(code, name, pocketpy::py_CompileMode::EXEC_MODE, None)
}

/// Create a new module and load it with code.
unsafe fn new_module(code: &str, name: &str) {
    let cname = create_raw_string!(name);
    let module = unsafe { pocketpy::py_getmodule(cname) };
    if !module.is_null() {
        panic!("module: {} already exists.", name);
    }

    let _ = unsafe { pocketpy::py_newmodule(cname) };

    // Exec the code
    let err = exec_py(code, format!("<{}>", name).as_str(), name);
    if !err.is_empty() {
        panic!("Setting new module error: {err}");
    }

    unsafe {
        free_raw_string!(cname);
    }
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
    if let Some(names) = state.defined_objects.borrow_mut().get_mut(&t) {
        names.insert(name.to_string());
    } else {
        let mut set = HashSet::new();
        set.insert(name.to_string());
        state.defined_objects.borrow_mut().insert(*t, set);
    }
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

/// Make a name private by prefixing "_pxs_"
pub(self) fn make_private(name: &str) -> String {
    format!("_pxs_{}", name)
}

/// Add your own private prefix after "_pxs_"
pub(self) fn make_private_prefix(name: &str, prefix: &str) -> String {
    make_private(format!("{prefix}_{name}").as_str())
}

/// This is the import overrider
unsafe extern "C" fn import_file(
    arg1: *const std::ffi::c_char,
    _data_size: *mut std::ffi::c_int,
) -> *mut std::ffi::c_char {
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
            // TODO: check this actually works dayo
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

/// Keep a reference to a python object/function.
pub(self) fn python_pxs_new_register(obj_ref: pocketpy::py_Ref) -> i32 {
    // Nothing to add
    if obj_ref.is_null() {
        return -1;
    }
    // Call _pxs_new_register
    unsafe {
        // Name
        let mname = create_raw_string!("_pxs_new_register");
        // Get method
        let method = pocketpy::py_getglobal(pocketpy::py_name(mname));
        if method.is_null() {
            panic!("_pxs_new_register is null in pocketpy");
        }

        // Push
        pocketpy::py_push(method);
        // Push self (null)
        pocketpy::py_pushnil();
        // Push args (obj_ref)
        let arg = pocketpy::py_pushtmp();
        py_assign(arg, obj_ref);

        //Call vector
        let ok = pocketpy::py_vectorcall(1, 0);
        if !ok {
            #[allow(unused)]
            let err = consume_error();
            pxs_debug!("Err: {err}");
            // TODO: exception
            return -1;
        }
        free_raw_string!(mname);

        let id = pocketpy::py_toint(pocketpy::py_retval());
        id as i32
    }
}

/// Get a pocketpy ref from a register
pub(self) fn python_pxs_get_register(idx: i32) -> bool {
    unsafe {
        let register_name = create_raw_string!("_pxs_register");
        let register = pocketpy::py_getglobal(pocketpy::py_name(register_name));
        free_raw_string!(register_name);
        if register.is_null() {
            return false;
        } else {
            let res = pocketpy::py_dict_getitem_by_int(register, idx as i64);
            if res == -1 {
                #[allow(unused)]
                // consume err
                let err = consume_error();
                pxs_debug!("Error in pxs_get_register: {err}");
                false
            } else {
                true
            }
        }
    }
}

/// Remove a reference from the _pxs_register
pub(self) fn python_pxs_remove_ref(idx: i32) {
    // Nothing to remove
    if idx == -1 {
        return;
    }
    unsafe {
        let register_name = create_raw_string!("_pxs_register");
        let register = pocketpy::py_getglobal(pocketpy::py_name(register_name));
        free_raw_string!(register_name);
        assert!(!register.is_null(), "_pxs_register must not be null");
        if register.is_null() {
            return;
        }

        let res = pocketpy::py_dict_delitem_by_int(register, idx as i64);
        // If error, consume it
        if res == -1 {
            #[allow(unused)]
            let err = consume_error();
            pxs_debug!("Error in pxs_remove_ref: {err}");
        }
    }
}

/// Do some python setup. This needs to be called for every thread too
unsafe fn python_setup() {
    unsafe {
        setup_module_loader();
    }

    // Setup some python code
    // 1. _pxs_new_register: registers a new object/function in our internal memory.
    let mut python_code = String::new();
    python_code.push_str(include_str!("../../core/python/main.py"));

    // with_feature!("pxs_utils", {
    //     // Set a new function (_pxs_items)
    //     python_code.push_str(include_str!("../../core/python/pxs_utils.py"));
    // });

    with_feature!("pxs_json", {
        // Create module
        unsafe {
            new_module(include_str!("../../core/python/pxs_json.py"), "pxs_json");
        }
        // Import into main
        python_code.push_str("\nimport pxs_json\n");
    });

    let res = exec_main_py(&python_code, "<python_setup>");
    if !res.is_empty() {
        panic!("Python setup error: {res}");
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
            python_setup();
            // setup_module_loader();
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
        create_module(&source);
    }

    fn execute(code: &str, file_name: &str) -> Result<pxs_Var> {
        let res = exec_main_py(code, file_name);
        if res.is_empty() {
            Ok(pxs_Var::new_null())
        } else {
            Ok(pxs_Var::new_exception(res))
        }
        // res
    }

    fn start_thread() {
        unsafe {
            let idx = pocketpy::py_currentvm() + 1;
            pocketpy::py_switchvm(idx);
            python_setup();
            // setup_module_loader();
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

    fn eval(code: &str) -> Result<pxs_Var> {
        let res = exec_main_py(code, "eval");
        Ok(if res.is_empty() {
            pocketpyref_to_var(unsafe { pocketpy::py_retval() })
        } else {
            pxs_Var::new_string(res)
        })
    }

    fn compile(code: &str, scope: pxs_Var) -> Result<pxs_Var> {
        let source = create_raw_string!(code);
        let file_name = create_raw_string!("<compile>");
        unsafe {
            let ok = pocketpy::py_compile(source, file_name, pocketpy::py_CompileMode::EXEC_MODE, true);
            free_raw_string!(source);
            free_raw_string!(file_name);
            if !ok {
                #[allow(unused)]
                let err = consume_error();
                // pxs_debug!("err: {err}");
                return Ok(pxs_Var::new_exception(err))
                // return pxs_Var::new_null();
            }
            let ud = pocketpy::py_retval();
            // To pxs
            let co = pocketpyref_to_var(ud);

            // Setup return
            let result = pxs_Var::new_list();
            let list = result.get_list().unwrap();
            // Push code object
            list.add_item(co);

            let tmp = pocketpy::py_pushtmp();
            // Check scope
            if scope.is_null() {
                // Empty dict
                pocketpy::py_newdict(tmp);
            } else if scope.is_map() {
                // To dict
                var_to_pocketpyref(tmp, &scope, None);
            } else {
                // Unsupported
                pocketpy::py_pop();
                return Ok(pxs_Var::new_exception(format!("Unsupported scope for Python: {:#?}", scope)));
            }

            // Convert into pxs
            list.add_item(pocketpyref_to_var(tmp));
            // Pop ref
            pocketpy::py_pop();

            Ok(result)
        }
    }
    
    fn exec_object(code: pxs_Var, scope: pxs_Var) -> Result<pxs_Var> {
        // Check if a list or a regular obj
        let code_obj = unsafe{pocketpy::py_pushtmp()};
        let code_scope = unsafe{pocketpy::py_pushtmp()};
        let code_locals = unsafe{pocketpy::py_pushtmp()};
        // let code_obj: &mut PythonPointer;
        // let code_scope: Option<&mut PythonPointer>;
        if code.is_list() {
            // Ensure there are 3 elements (runtime, objectscope)
            let list = code.get_list().unwrap();
            if list.len() != 3 {
                // TODO: pxs_Error
                return Ok(pxs_Var::new_exception(format!("List length is not 3. Len: {}", list.len())));
            }

            // Ok let's get the object and scope
            let object = list.get_item(1).unwrap();
            if !object.is_object() {
                // TODO: pxs_Error
                return Ok(pxs_Var::new_exception("Code Object is not a object".to_string()));
            }
            let scope = list.get_item(2).unwrap();
            if !scope.is_object() {
                // TODO: pxs_Error
                return Ok(pxs_Var::new_exception("Scope is not a object".to_string()));
            }

            var_to_pocketpyref(code_obj, object, None);
            var_to_pocketpyref(code_scope, scope, None);
        } else {
            // TODO: pxs_Error
            return Ok(pxs_Var::new_exception("Compiled code is not a list".to_string()));
        }

        // Check if a optional scope
        if scope.is_map() {
            // Setup
            var_to_pocketpyref(code_locals, &scope, None);
        } else {
            unsafe{
                pocketpy::py_newdict(code_locals);
            }
        }

        unsafe {
            // Get exec function
            let exec_func = get_builtin("exec");
            if exec_func.is_null() {
                return Ok(pxs_Var::new_exception("Could not find `exec` in Python builtins".to_string()));
            }
            pocketpy::py_push(exec_func);
            pocketpy::py_pushnil();
            pocketpy::py_push(code_obj);
            pocketpy::py_push(code_scope);
            pocketpy::py_push(code_locals);
            let ok = pocketpy::py_vectorcall(3, 0);
            // Pop tmps created by me.
            pocketpy::py_pop();
            pocketpy::py_pop();
            pocketpy::py_pop();
            if !ok {
                let err = consume_error();
                return Ok(pxs_Var::new_exception(err));
            }

            Ok(pocketpyref_to_var(pocketpy::py_retval()))            
        }
    }
}

/// Add pxs vars to the stack
fn add_args(args: &Vec<pxs_Var>) {
    // Convert args into py_Ref
    for i in 0..args.len() {
        let pyref = unsafe { pocketpy::py_pushtmp() };
        var_to_pocketpyref(pyref, &args[i], None);
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
        var_to_pocketpyref(obj_ref, var, None);

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
                // TODO: exception.
                let err = consume_error();
                return Ok(pxs_Var::new_exception(err));
            }

            let final_var = pocketpyref_to_var(pocketpy::py_retval());

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
                    // Look for in current module
                    let cmod = pocketpy::py_inspect_currentmodule();
                    // TODO: does cmod need to be null checked.
                    // Then look for a method in current module
                    let found = pocketpy::py_getattr(cmod, pymethod_name);
                    if !found {
                        // Consume the error (we don't care about it anymore)
                        let _ = consume_error();
                        // Check in __main__
                        let found = pocketpy::py_getglobal(pymethod_name);
                        if found.is_null() {
                            pxs_debug!(
                                "Function: {method}, not found in globals, current mod, OR __main__."
                            );
                        }

                        found
                    } else {
                        pocketpy::py_retval()
                    }
                }
            };
            free_raw_string!(method_name);

            if pymethod.is_null() {
                return Ok(pxs_Var::new_exception(format!(
                    "Method: {method} was not found"
                )));
            }

            // Push method
            pocketpy::py_push(pymethod);
            // Push self, in this case nil.
            pocketpy::py_pushnil();

            add_args(&args.vars);

            // Call it via vectrocall
            let ok = pocketpy::py_vectorcall(args.vars.len() as u16, 0);
            if !ok {
                let err = consume_error();
                return Ok(pxs_Var::new_exception(err));
            }

            // Guard the stack
            let final_var = pocketpyref_to_var(pocketpy::py_retval());
            Ok(final_var)
        }
    }

    fn var_call(method: &pxs_Var, args: &mut pxs_VarList) -> Result<pxs_Var, anyhow::Error> {
        pxs_debug!("PYTHON VAR CALL IS GETTING CALLED");
        // Make sure it's a function!
        if !method.is_function() {
            return Err(anyhow!("Expected Function, found: {:#?}", method.tag));
        }

        // Get ptr as py_ref
        let fn_ptr = unsafe { PythonPointer::from_borrow_void(method.get_function().unwrap()) };
        let pyfn = fn_ptr.get_ptr();

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
            let err = consume_error();
            // pxs_debug!("calling function failed. Error: {err}");
            return Ok(pxs_Var::new_exception(err));
        }

        let py_res = unsafe { pocketpy::py_retval() };
        Ok(pocketpyref_to_var(py_res))
    }

    fn get(var: &pxs_Var, key: &str) -> Result<pxs_Var, anyhow::Error> {
        unsafe {
            if var.value.object_val.is_null() {
                return Err(anyhow!("var.value.object_val is Null"));
            }
            // Deref
            let python_pointer = PythonPointer::from_borrow_void(var.value.object_val);
            let object = python_pointer.get_ptr();
            let raw_key = create_raw_string!(key);
            let py_key = pocketpy::py_name(raw_key);
            free_raw_string!(raw_key);
            let res = pocketpy::py_getattr(object, py_key);

            if !res {
                return Ok(pxs_Var::new_exception(consume_error()));
            }

            let py_res = pocketpy::py_retval();
            // Get value
            Ok(pocketpyref_to_var(py_res))
        }
    }

    fn set(var: &pxs_Var, key: &str, value: &pxs_Var) -> Result<pxs_Var, anyhow::Error> {
        unsafe {
            if var.value.object_val.is_null() {
                return Err(anyhow!("var.value.object_val is Null"));
            }

            // Deref
            let object = PythonPointer::from_borrow_void(var.value.object_val).get_ptr();
            // Key
            let raw_key = create_raw_string!(key);
            let py_key = pocketpy::py_name(raw_key);
            free_raw_string!(raw_key);
            // Set
            let tmp = pocketpy::py_pushtmp();
            var_to_pocketpyref(tmp, value, None);
            let res = pocketpy::py_setattr(object, py_key, tmp);

            if !res {
                return Ok(pxs_Var::new_exception(consume_error()));
            }

            Ok(pocketpyref_to_var(pocketpy::py_retval()))
        }
    }

    fn get_from_name(name: &str) -> Result<pxs_Var, anyhow::Error> {
        unsafe {
            let ref_name = create_raw_string!(name);
            let pyname = pocketpy::py_name(ref_name);
            let py_ref = {
                // Try a builtin first
                let global = pocketpy::py_getbuiltin(pyname);
                if !global.is_null() {
                    global
                } else {
                    // Look for in current module
                    let cmod = pocketpy::py_inspect_currentmodule();
                    // TODO: does cmod need to be null checked.
                    // Then look for a ref in current module
                    let found = pocketpy::py_getattr(cmod, pyname);
                    if !found {
                        // Consume the error (we don't care about it anymore)
                        let _ = consume_error();
                        // Check in __main__
                        let found = pocketpy::py_getglobal(pyname);
                        if found.is_null() {
                            pxs_debug!(
                                "Ref: {name}, not found in globals, current mod, OR __main__."
                            );
                        }

                        found
                    } else {
                        pocketpy::py_retval()
                    }
                }
            };
            free_raw_string!(ref_name);
            Ok(pocketpyref_to_var(py_ref))
        }
    }
}
