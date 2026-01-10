use std::{cell::RefCell, collections::HashMap, sync::{Arc, OnceLock}};

use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use rustpython::vm::{Interpreter, PyObjectRef, PyRef, Settings, VirtualMachine, builtins::PyType, convert::ToPyObject, function::FsPath, scope::Scope};

use crate::{python::{func::create_function, module::create_module, overrides::override_import_loader}, shared::{PixelScript, read_file}};

mod var;
mod func;
mod module;
mod object;
mod overrides;

/// This is the Python State
struct State {
    /// The actual Python Interpreter
    engine: Interpreter,
    /// The global variable scope (for running in __main__)
    global_scope: PyObjectRef,
    /// Cached class types
    class_types: RefCell<HashMap<String, &'static PyRef<PyType>>>,
    /// Cached class ptrs
    class_ptrs: RefCell<Vec<*mut PyRef<PyType>>>,
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
pub(self) fn get_class_type_from_cache(type_name: &str) -> Option<&'static PyRef<PyType>> {
    let state = get_state();
    state.class_types.borrow().get(type_name).cloned()
}

/// Store a new class type in cache.
pub(self) fn store_class_type_in_cache(type_name: &str, class_type: PyRef<PyType>) {
    let state = get_state();

    // Leak class
    let class_static: &'static PyRef<PyType> = unsafe {
        let leaked_ptr = Box::into_raw(Box::new(class_type.clone()));
        state.class_ptrs.borrow_mut().push(leaked_ptr);
        // Cache ptr
        &*leaked_ptr // Dereference to get the static ref
    };
    state.class_types.borrow_mut().insert(type_name.to_string(), class_static);
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

        for ptr in self.class_ptrs.borrow().iter() {
            if !ptr.is_null() {
                unsafe {
                    let _ = Box::from_raw(ptr.to_owned());
                }
            }
        }
        self.class_ptrs.borrow_mut().clear();
    }
}
unsafe impl Send for State {}
unsafe impl Sync for State {}

/// The State static variable for Lua.
static STATE: OnceLock<ReentrantMutex<State>> = OnceLock::new();

/// Get the state of Python.
fn get_state() -> ReentrantMutexGuard<'static, State> {
    let mutex = STATE.get_or_init(|| {
        // Initialize state inside
        let mut settings = Settings::default();
        settings.path_list.push("".to_string());
        settings.write_bytecode = false;
        
        let interp = rustpython::InterpreterConfig::new()
            .settings(settings)
            .init_stdlib() 
            .interpreter();

        let scope = interp.enter(|vm| {
            let globals = vm.ctx.new_dict();

            // let sys_modules = vm.sys_module.get_attr("modules", vm).unwrap();
            
            // let modules_dict = sys_modules.downcast::<rustpython::vm::builtins::PyDict>().unwrap();
            
            // Remove dangerous modules from the cache so 'import os' fails
            // let _ = modules_dict.del_item("os", vm);
            // let _ = modules_dict.del_item("io", vm);
            // let _ = modules_dict.del_item("shutil", vm);

            globals.into()
        });
        ReentrantMutex::new(
            State { 
                engine: interp, 
                global_scope: scope,
                class_types: RefCell::new(HashMap::new()),
                cached_leaks: RefCell::new(HashMap::new()),
                class_ptrs: RefCell::new(vec![])
            }
        )
    });

    mutex.lock()
}

pub struct PythonScripting {}

impl PixelScript for PythonScripting {
    fn start() {
        // Initalize the state
        let state = get_state();
        state.engine.enter(|vm| {
            override_import_loader(vm, state.global_scope.clone());
        });
    }

    fn stop() {
        // Nothing is really needed to be done here? Except maybe do some GC?
    }

    fn add_variable(name: &str, variable: &crate::shared::var::Var) {
        let state = get_state();
        state.engine.enter(|vm| {
            let var = variable.clone().to_pyobject(vm);
            vm.builtins.set_attr(unsafe {pystr_leak( name.to_string()) }, var, vm).expect("Could not set Var Python.");
        });
    }

    fn add_callback(name: &str, fn_idx: i32) {
        let state = get_state();
        state.engine.enter(|vm| {
            let pyfunc = create_function(vm, name, fn_idx);
            // Attach it
            vm.builtins.set_attr(unsafe { pystr_leak(name.to_string()) }, pyfunc, vm).expect("Could not set callback Python.");
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

// TODO: this but for Python

// impl ObjectMethods for LuaScripting {
//     fn object_call(
//         var: &crate::shared::var::Var,
//         method: &str,
//         args: Vec<crate::shared::var::Var>,
//     ) -> Result<crate::shared::var::Var, anyhow::Error> {
//         // Get the lua table.
//         let table = unsafe {
//             if var.is_host_object() {
//                 // This is from the PTR!
//                 let pixel_object = get_object(var.value.host_object_val).expect("No HostObject found.");
//                 let lang_ptr = pixel_object.lang_ptr.lock().unwrap();
//                 // Get as table.
//                 let table_ptr = *lang_ptr as *const LuaTable;
//                 // Return table
//                 (&*table_ptr).clone()
//             } else {
//                 // Just grab it from the ptr itself
//                 let table_ptr = var.value.object_val as *const LuaTable;
//                 (&*table_ptr).clone()
//             }
//         };

//         // Call method
//         let mut lua_args = vec![];
//         {
//             // State start
//             let state = get_lua_state();
//             for arg in args {
//                 lua_args.push(arg.into_lua(&state.engine).expect("Could not convert Var into Lua Var"));
//             }
//             // State drop
//         }
//         // The function could potentially call the state
//         let res = table.call_function(method, lua_args).expect("Could not call function on Lua Table.");

//         // State start again
//         let state = get_lua_state();
//         let pixel_res = Var::from_lua(res, &state.engine).expect("Could not convert LuaVar into PixelScript Var.");

//         Ok(pixel_res)
//         // Drop state
//     }
// }
