// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
pub mod func;
pub mod module;
pub mod object;
pub mod var;

use anyhow::anyhow;
use mlua::prelude::*;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use std::{cell::RefCell, collections::HashMap};

use crate::{
    lua::var::{from_lua, into_lua},
    shared::{PixelScript, object::get_object, read_file, var::{ObjectMethods, pxs_Var}},
};

thread_local! {
    static LUASTATE: ReentrantMutex<State> = ReentrantMutex::new(init_state());
}

/// This is the Lua state. Each language gets it's own private state
struct State {
    /// The lua engine.
    engine: Lua,
    /// Cached Tables
    tables: RefCell<HashMap<String, LuaTable>>,
    /// Boxed Tables
    boxed_tables: RefCell<Vec<Box<*mut LuaTable>>>,
    /// Boxed Functions
    boxed_functions: RefCell<Vec<Box<*mut LuaFunction>>>
}

/// Initialize Lua state per thread.
fn init_state() -> State {
    State {
        engine: Lua::new(),
        tables: RefCell::new(HashMap::new()),
        boxed_tables: RefCell::new(Vec::new()),
        boxed_functions: RefCell::new(Vec::new()),
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
    let package: LuaTable = lua
        .globals()
        .get("package")
        .expect("Could not get package Lua.");
    let searchers: LuaTable = package
        .get("searchers")
        .expect("Could not get searchers Lua.");

    // Custom loader function
    let loader = lua
        .create_function(|lua, name: String| {
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
        })
        .expect("Could not create loader function Lua.");

    // Set our loader in searchers list
    let len = searchers
        .len()
        .expect("Could not get len of searchers Lua.");
    searchers
        .set(len + 1, loader)
        .expect("Could not set loader in searchers Lua.");
}

pub struct LuaScripting;

impl PixelScript for LuaScripting {
    fn add_module(source: std::sync::Arc<crate::shared::module::pxs_Module>) {
        module::add_module(source, None);
    }

    fn execute(code: &str, file_name: &str) -> String {
        execute(code, file_name)
    }

    fn start() {
        // Initalize the state
        let state = get_lua_state();
        setup_module_loader(&state.engine);
    }

    fn stop() {
        // Kill lua
        let state = get_lua_state();

        // First drop the boxed stuff
        for item in state.boxed_functions.borrow().iter() {
            let _ = unsafe { Box::from_raw(**item) };
        }
        for item in state.boxed_tables.borrow().iter() {
            let _ = unsafe { Box::from_raw(**item) };
        }

        // Ok clear the cached tables
        state.tables.borrow_mut().clear();

        // Ok now cler the GC.
        state.engine.gc_collect().unwrap();
        state.engine.gc_collect().unwrap();
    }

    fn start_thread() {
        // LUA does not need this.
    }

    fn stop_thread() {
        // Run the stop logic.
        Self::stop();
    }
}

/// Convert args for ObjectMethods into LuaMutliValue
fn args_to_lua(args: &Vec<pxs_Var>) -> LuaMultiValue {
    let mut lua_args = vec![];
    let state = get_lua_state();
    for arg in args.iter() {
        lua_args.push(into_lua(&state.engine, arg).expect("Could not convert Var into Lua Var"));
    }

    // Pack lua args
    LuaMultiValue::from_vec(lua_args)
}

impl ObjectMethods for LuaScripting {
    fn object_call(
        var: &crate::shared::var::pxs_Var,
        method: &str,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        // Get the lua table.
        let table = unsafe {
            if var.is_host_object() {
                // This is from the PTR!
                let pixel_object =
                    get_object(var.value.host_object_val).expect("No HostObject found.");
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

        let lua_args = args_to_lua(&args.vars);
        let res = table
            .call_function(method, lua_args)
            .expect("Could not call function on Lua Table.");

        let pixel_res = from_lua(res).expect("Could not convert LuaVar into PixelScript Var.");

        Ok(pixel_res)
        // Drop state
    }

    fn call_method(
        method: &str,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        // Get args as lua args
        let lua_args = args_to_lua(&args.vars);
        let state = get_lua_state();

        let function: LuaFunction = state.engine.globals().get(method)?;
        let res: LuaValue = function
            .call(lua_args)
            .expect("Could not call Lua method.");

        from_lua(res)
    }

    fn var_call(
        method: &crate::shared::var::pxs_Var,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        if !method.is_function() {
            return Err(anyhow!("Expected a Function, found a: {:#?}", method.tag));
        }

        // Get the pointer and convert it into a LuaFunction
        let fn_ptr = method.get_function().unwrap();
        let lua_function = fn_ptr as *const LuaFunction;

        // Convert  the methods into lua args
        let lua_args = args_to_lua(&args.vars);

        // Call function
        let res: LuaValue = (unsafe { &*lua_function }).call(lua_args).expect("Could not call Lua method.");

        // Convert into pxs
        from_lua(res)
    }
}
