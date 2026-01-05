use std::sync::{Mutex, OnceLock};

use rustpython::vm::{Interpreter, scope::Scope};

use crate::shared::PixelScript;

mod var;
mod func;
mod module;
mod object;

/// This is the Python State
struct State {
    engine: Interpreter,
    global_scope: Scope
}

/// The State static variable for Lua.
static STATE: OnceLock<Mutex<State>> = OnceLock::new();

/// Get the state of LUA.
fn get_state() -> std::sync::MutexGuard<'static, State> {
    let interp = rustpython::InterpreterConfig::new().interpreter();
    let scope = interp.enter(|vm| {
        let globals = vm.ctx.new_dict();

        Scope::new(None, globals)
    });
    let mutex = STATE.get_or_init(|| Mutex::new(State { engine: interp, global_scope: scope }));

    // This will block the C thread if another thread is currently using Lua
    mutex.lock().expect("Failed to lock Python State")
}

pub struct PythonScripting {}

impl PixelScript for PythonScripting {
    fn start() {
        // Initalize the state
        let _ununsed = get_state();
    }

    fn stop() {
        // TODO: Stop python
        let _state = get_state();
    }

    fn add_variable(name: &str, variable: &crate::shared::var::Var) {
        let state = get_state();
        state.engine.enter(|vm| {
            let t = vm.ctx.new_str("s");
            state.global_scope.locals.set_item("t", t.into(), vm).expect("Could not set");
        });
        // todo!()
    }

    fn add_object_variable(name: &str, idx: i32) {
        todo!()
    }

    fn add_callback(name: &str, fn_idx: i32) {
        todo!()
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::Module>) {
        todo!()
    }

    fn add_object(name: &str, callback: crate::shared::func::Func, opaque: *mut std::ffi::c_void) {
        todo!()
    }

    fn execute(code: &str, file_name: &str) -> String {
        todo!()
    }
}