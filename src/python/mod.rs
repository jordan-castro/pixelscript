use std::{cell::RefCell, collections::HashMap, sync::{Arc, OnceLock}};

use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use rustpython::vm::{Interpreter, PyObjectRef, Settings, convert::ToPyObject, scope::Scope};

use crate::{python::{func::create_function, module::create_module}, shared::PixelScript};

mod var;
mod func;
mod module;
mod object;

/// This is the Python State
struct State {
    /// The actual Python Interpreter
    engine: Interpreter,
    /// The global variable scope (for running in __main__)
    global_scope: PyObjectRef,
    /// Cached class types
    class_types: RefCell<HashMap<String, PyObjectRef>>,
    /// Cached leaked names.
    cached_leaks: RefCell<HashMap<String,  *mut str>>
}

/// Create a string for Python enviroment. String is cached and will be freed later automatically.
pub(self) unsafe fn pystr_leak(s:String) -> &'static str {
    let state = get_state();
    if let Some(&ptr) = state.cached_leaks.borrow().get(&s) {
        return unsafe {&*ptr};
    }

    // Convert to box
    let b = s.clone().into_boxed_str();
    let ptr = Box::into_raw(b);

    // Store in cache
    state.cached_leaks.borrow_mut().insert(s, ptr);

    // Return as static reference
    unsafe {&*ptr}
}

/// Get a class type from cache
pub(self) fn get_class_type_from_cache(type_name: &str) -> Option<PyObjectRef> {
    let state = get_state();
    state.class_types.borrow().get(type_name).cloned()
}

/// Store a new class type in cache.
pub(self) fn store_class_type_in_cache(type_name: &str, class_type: PyObjectRef) {
    let state = get_state();
    state.class_types.borrow_mut().insert(type_name.to_string(), class_type);
}

impl Drop for State {
    fn drop(&mut self) {
        self.class_types.borrow_mut().clear();

        for (_, ptr) in self.cached_leaks.borrow_mut().drain() {
            if !ptr.is_null() {
                unsafe {
                    let _ = Box::from_raw(ptr);
                }
            }
        }
    }
}
unsafe impl Send for State {}
unsafe impl Sync for State {}

/// The State static variable for Lua.
static STATE: OnceLock<ReentrantMutex<State>> = OnceLock::new();

/// Get the state of LUA.
fn get_state() -> ReentrantMutexGuard<'static, State> {
    let mutex = STATE.get_or_init(|| {
        // Initialize state inside
        let mut settings = Settings::default();
        settings.path_list.push("".to_string());
        let interp = rustpython::InterpreterConfig::new()
            .settings(settings)
            .init_stdlib() 
            .interpreter();
        
        let scope = interp.enter(|vm| {
            let globals = vm.ctx.new_dict();
            let sys_modules = vm.sys_module.get_attr("modules", vm).unwrap();
            
            let modules_dict = sys_modules.downcast::<rustpython::vm::builtins::PyDict>().unwrap();
            
            // Remove dangerous modules from the cache so 'import os' fails
            let _ = modules_dict.del_item("os", vm);
            let _ = modules_dict.del_item("io", vm);
            let _ = modules_dict.del_item("shutil", vm);

            globals.into()
        });
        ReentrantMutex::new(
            State { 
                engine: interp, 
                global_scope: scope,
                class_types: RefCell::new(HashMap::new()),
                cached_leaks: RefCell::new(HashMap::new())
            }
        )

    });

    mutex.lock()
}

pub struct PythonScripting {}

impl PixelScript for PythonScripting {
    fn start() {
        // Initalize the state
        let _ununsed = get_state();
    }

    fn stop() {
        // Nothing is really needed to be done here? Except maybe do some GC?
    }

    fn add_variable(name: &str, variable: &crate::shared::var::Var) {
        let state = get_state();
        state.engine.enter(|vm| {
            let var = variable.clone().to_pyobject(vm);
            state.global_scope.set_item(name, var.into(), vm).expect("Could not set");
        });
    }

    fn add_callback(name: &str, fn_idx: i32) {
        let state = get_state();
        state.engine.enter(|vm| {
            let pyfunc = create_function(vm, name, fn_idx);
            // Attach it
            state.global_scope.set_item(name, pyfunc.into(), vm).expect("Could not set");
        });
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::Module>) {
        let state = get_state();
        state.engine.enter(|vm| {
            create_module(vm, Arc::clone(&source));
        });
    }

    fn execute(code: &str, file_name: &str) -> String {
        let state = get_state();
        state.engine.enter(|vm| {
            let dict = state.global_scope.clone().downcast::<rustpython::vm::builtins::PyDict>().expect("Could not downcast to Dict, Python.");
            let scope = Scope::with_builtins(None, dict, vm);

            match vm.run_code_string(scope, code, file_name.to_string()) {
                Ok(_) => {
                    String::from("")
                },
                Err(e) => {
                    e.to_pyobject(vm).str(vm).expect("Could not get error string Python.").as_str().to_string()
                },
            }
        })
    }
}