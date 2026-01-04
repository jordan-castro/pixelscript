pub mod object;
pub mod func;
pub mod module;
pub mod var;

use mlua::prelude::*;
use std::sync::{Arc, Mutex, OnceLock};

use crate::{lua::object::create_object, shared::{PixelScript, object::{PixelObject, get_object_lookup}, var::{ObjectMethods, Var}}};

/// This is the Lua state. Each language gets it's own private state
struct State {
    /// The lua engine.
    engine: Lua,
}

/// The State static variable for Lua.
static STATE: OnceLock<Mutex<State>> = OnceLock::new();

/// Get the state of LUA.
fn get_state() -> std::sync::MutexGuard<'static, State> {
    let mutex = STATE.get_or_init(|| Mutex::new(State { engine: Lua::new() }));

    // This will block the C thread if another thread is currently using Lua
    mutex.lock().expect("Failed to lock Lua State")
}

/// Execute some orbituary lua code.
/// Returns a String. Empty means no error happened and was successful!
pub fn execute(code: &str, file_name: &str) -> String {
    let state = get_state();
    let res = state.engine.load(code).exec();
    if res.is_err() {
        let error_str = format!(
            "Error in LUA: {}, for file: {}",
            res.unwrap_err().to_string(),
            file_name
        );
        return error_str;
    }

    String::from("")
}

pub struct LuaScripting {}

impl PixelScript for LuaScripting {
    fn add_variable(name: &str, variable: &crate::shared::var::Var) {
        var::add_variable(&get_state().engine, name, variable.clone());
    }

    fn add_callback(
        name: &str,
        callback: crate::shared::func::Func,
        opaque: *mut std::ffi::c_void,
    ) {
        func::add_callback(name, callback, opaque);
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::Module>) {
        module::add_module(source);
    }

    fn execute(code: &str, file_name: &str) -> String {
        execute(code, file_name)
    }

    fn add_object_variable(name: &str, source: Arc<PixelObject>) {
        // Get new IDX.
        let mut object_lookup = get_object_lookup();
        let idx = object_lookup.add_object(Arc::clone(&source));
        // Pass just the idx into the variable... This is a interesting one....
        LuaScripting::add_variable(name, &&Var::new_host_object(idx));
    }

    fn start() {
        // Initalize the state
        let _ununsed = get_state();
    }
    
    fn stop() {
        // Kill lua
        let state = get_state();

        state.engine.gc_collect().unwrap();
        state.engine.gc_collect().unwrap();
    }
}

impl ObjectMethods for LuaScripting {
    fn object_call(
        &self,
        var: &crate::shared::var::Var,
        method: &str,
        args: Vec<crate::shared::var::Var>,
    ) -> Result<crate::shared::var::Var, anyhow::Error> {
        // Get the lua table.
        let table = unsafe {
            if var.is_host_object() {
                let object_lookup = get_object_lookup();
                // This is from the PTR!
                let pixel_object = object_lookup.get_object(var.value.host_object_val).expect("No HostObject found.");
                let lang_ptr = pixel_object.lang_ptr.lock().unwrap();
                // Get as table.
                let table_ptr = *lang_ptr as *const LuaTable;
                // Return table
                (&*table_ptr).clone()
            } else {
                // Just grab it from the ptr itself
                let table_ptr = var.value.object_val as *const LuaTable;
                (&*table_ptr).clone()
            }
        };

        let state = get_state();
        // Call method
        let mut lua_args = vec![];
        for arg in args {
            lua_args.push(arg.into_lua(&state.engine).expect("Could not convert Var into Lua Var"));
        }
        let res = table.call_function(method, lua_args).expect("Could not call function on Lua Table.");

        let pixel_res = Var::from_lua(res, &state.engine).expect("Could not convert LuaVar into PixelScript Var.");

        Ok(pixel_res)
    }
}
