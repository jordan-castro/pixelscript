pub mod var;
pub mod func;

use std::{collections::HashMap, sync::{Mutex, OnceLock}};
use mlua::prelude::*;

use crate::shared::func::Func;

/// This is the Lua state. Each language gets it's own private state
struct State {
    /// The lua engine.
    engine: Lua,
}

/// The State static variable for Lua.
static STATE: OnceLock<Mutex<State>> = OnceLock::new();

/// Get the state of LUA.
fn get_state() -> std::sync::MutexGuard<'static, State> {
    let mutex = STATE.get_or_init(|| {
        Mutex::new(State {
            engine: Lua::new(),
        })
    });
    
    // This will block the C thread if another thread is currently using Lua
    mutex.lock().expect("Failed to lock Lua State")
}
/// Execute some orbituary lua code.
/// Returns a String. Empty means no error happened and was successful!
pub fn execute(code: &str, file_name: &str) -> String {
    let state = get_state();
    let res = state.engine.load(code).exec();
    if res.is_err() {
        return res.unwrap_err().to_string();
    }

    String::from("")
}