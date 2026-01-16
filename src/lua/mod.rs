// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
pub mod object;
pub mod func;
pub mod module;
pub mod var;

use mlua::prelude::*;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use std::{cell::RefCell, collections::HashMap};

use crate::{lua::var::{from_lua, into_lua}, shared::{PixelScript, object::get_object, read_file, var::{ObjectMethods}}};

thread_local! {
    static LUASTATE: ReentrantMutex<State> = ReentrantMutex::new(init_state());
}

/// This is the Lua state. Each language gets it's own private state
struct State {
    /// The lua engine.
    engine: Lua,
    /// Cached Tables
    tables: RefCell<HashMap<String, LuaTable>>
}

/// Initialize Lua state per thread.
fn init_state() -> State {
    State {
        engine: Lua::new(),
        tables: RefCell::new(HashMap::new()),
    }
}

/// Get the state of LUA.
fn get_lua_state() -> ReentrantMutexGuard<'static, State> {
    LUASTATE.with(|mutex| {
        let guard = mutex.lock();
        // Transmute the lifetime so the guard can be passed around the thread
        unsafe { std::mem::transmute(guard) }
    })
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
    // fn add_variable(name: &str, variable: &crate::shared::var::Var) {
    //     var::add_variable(&get_lua_state().engine, name, variable);
    // }

    // fn add_callback(
    //     name: &str,
    //     fn_idx: i32
    // ) {
    //     func::add_callback(name, fn_idx);
    // }

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
    
    fn start_thread() {
        // LUA does not need this.
    }
    
    fn stop_thread() {
        // LUA does not need this.
    }
}

impl ObjectMethods for LuaScripting {
    fn object_call(
        var: &crate::shared::var::Var,
        method: &str,
        args: &Vec<crate::shared::var::Var>,
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
            for arg in args.iter() {
                lua_args.push(into_lua(&state.engine, arg).expect("Could not convert Var into Lua Var"));
            }
            // State drop
        }
        // The function could potentially call the state
        let res = table.call_function(method, lua_args).expect("Could not call function on Lua Table.");

        let pixel_res = from_lua(res).expect("Could not convert LuaVar into PixelScript Var.");

        Ok(pixel_res)
        // Drop state
    }
}
