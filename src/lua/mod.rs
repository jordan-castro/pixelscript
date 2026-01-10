pub mod object;
pub mod func;
pub mod module;
pub mod var;

use mlua::prelude::*;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use std::{cell::RefCell, collections::HashMap, sync::{OnceLock}};

use crate::shared::{PixelScript, object::get_object, read_file, var::{ObjectMethods, Var}};

/// This is the Lua state. Each language gets it's own private state
struct State {
    /// The lua engine.
    engine: Lua,
    /// Cached Tables
    tables: RefCell<HashMap<String, LuaTable>>
}

/// The State static variable for Lua.
static STATE: OnceLock<ReentrantMutex<State>> = OnceLock::new();

/// Get the state of LUA.
fn get_lua_state() -> ReentrantMutexGuard<'static, State> {
    let mutex = STATE.get_or_init(|| {
        ReentrantMutex::new(State { 
            engine: Lua::new(), 
            tables: RefCell::new(HashMap::new()) 
        })
    });
    // This will 
    mutex.lock()
}

/// Get a cached metatable from lua.
pub(self) fn get_metatable(name: &str) -> Option<LuaTable> {
    let state = get_lua_state();
    state.tables.borrow().get(name).cloned()
}

/// Cahce a metatable.
pub(self) fn store_metatable(name: &str, table: LuaTable) {
    let state = get_lua_state();
    state.tables.borrow_mut().insert(name.to_string(), table);
}

/// Execute some orbituary lua code.
/// Returns a String. Empty means no error happened and was successful!
pub fn execute(code: &str, file_name: &str) -> String {
    let res = {
        let state = get_lua_state();
        state.engine.load(code).exec()
    };
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

/// Custom moduile loader function
fn setup_module_loader(lua: &Lua) {
    // Get package.searchers
    let package : LuaTable = lua.globals().get("package").expect("Could not get package Lua.");
    let searchers: LuaTable = package.get("searchers").expect("Could not get searchers Lua.");

    // Custom loader function
    let loader = lua.create_function(|lua, name: String| {
        let path = name.replace(".", "/");
        let path = if !path.ends_with(".lua") {
            format!("{path}.lua").to_string()
        } else {
            path
        };
        let contents = read_file(path.as_str());

        if contents.is_empty() {
            return Ok(LuaNil);
        }

        // Compile into chunk
        match lua.load(contents).set_name(&path).into_function() {
            Ok(func) => Ok(LuaValue::Function(func)),
            Err(_) => Ok(LuaNil),
        }
    }).expect("Could not create loader function Lua.");

    // Set our loader in searchers list
    let len = searchers.len().expect("Could not get len of searchers Lua.");
    searchers.set(len + 1, loader).expect("Could not set loader in searchers Lua.");
}

pub struct LuaScripting;

impl PixelScript for LuaScripting {
    fn add_variable(name: &str, variable: &crate::shared::var::Var) {
        var::add_variable(&get_lua_state().engine, name, variable.clone());
    }

    fn add_callback(
        name: &str,
        fn_idx: i32
    ) {
        func::add_callback(name, fn_idx);
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::Module>) {
        module::add_module(source, None);
    }

    fn execute(code: &str, file_name: &str) -> String {
        execute(code, file_name)
    }

    // fn add_object_variable(name: &str, idx: i32) {
    //     // Pass just the idx into the variable... This is a interesting one....
    //     LuaScripting::add_variable(name, &&Var::new_host_object(idx));
    // }

    fn start() {
        // Initalize the state
        let state = get_lua_state();
        setup_module_loader(&state.engine);
    }

    fn stop() {
        // Kill lua
        let state = get_lua_state();

        state.engine.gc_collect().unwrap();
        state.engine.gc_collect().unwrap();
    }

}

impl ObjectMethods for LuaScripting {
    fn object_call(
        var: &crate::shared::var::Var,
        method: &str,
        args: Vec<crate::shared::var::Var>,
    ) -> Result<crate::shared::var::Var, anyhow::Error> {
        // Get the lua table.
        let table = unsafe {
            if var.is_host_object() {
                // This is from the PTR!
                let pixel_object = get_object(var.value.host_object_val).expect("No HostObject found.");
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

        // Call method
        let mut lua_args = vec![];
        {
            // State start
            let state = get_lua_state();
            for arg in args {
                lua_args.push(arg.into_lua(&state.engine).expect("Could not convert Var into Lua Var"));
            }
            // State drop
        }
        // The function could potentially call the state
        let res = table.call_function(method, lua_args).expect("Could not call function on Lua Table.");

        // State start again
        let state = get_lua_state();
        let pixel_res = Var::from_lua(res, &state.engine).expect("Could not convert LuaVar into PixelScript Var.");

        Ok(pixel_res)
        // Drop state
    }
}
