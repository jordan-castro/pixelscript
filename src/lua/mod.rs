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
    lua::var::{from_lua, into_lua}, pxs_newexception, shared::{PixelScript, read_file, var::{ObjectMethods, pxs_Var}}, with_feature
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
}

macro_rules! get_lua_res {
    ($err:expr, $func:expr, $($arg:tt)*) => {{
        let res = $func($($arg)*);
        if res.is_err() {
            return pxs_Var::new_exception($err);
        }
        res.unwrap()
    }};
}

/// Preload a lua source code as a module.
fn preload_lua_module(lua: &Lua, code: &str, name: &str) -> Result<(), anyhow::Error> {
    let package: LuaTable = lua.globals().get("package")?;
    let preload: LuaTable = package.get("preload")?;

    let owned_code = String::from(code);
    let owned_name = String::from(name);

    let loader = lua.create_function(move |lua, _: ()| {
        let res: LuaTable = lua.load(&owned_code).set_name(format!("pxs_internal_{}", &owned_name)).eval()?;
        Ok(res)
    })?;

    preload.set(name, loader)?;
    Ok(())
}

/// Initialize Lua state per thread.
fn init_state() -> State {
    // Define a global function in engine
    let engine = Lua::new();

    let mut lua_globals = String::new();
    lua_globals.push_str(include_str!("../../core/lua/main.lua"));

    // with_feature!("pxs_utils", {
    //     // Load in the pxs_utils methods into GLOBAL scope.
    //     lua_globals.push_str(include_str!("../../core/lua/pxs_utils.lua"));
    // });

    with_feature!("pxs_json", {
        // Load dkjson module
        preload_lua_module(&engine, include_str!("../../libs/dkjson.lua"), "__dkjson__").expect("Could not load dkjson.lua");
        // Load in the pxs_json module
        preload_lua_module(&engine, include_str!("../../core/lua/pxs_json.lua"), "pxs_json").expect("Could not load pxs_json.lua");
        // Import it globally
        lua_globals.push_str("\npxs_json = require('pxs_json')\n");
    });
    engine.load(lua_globals).exec().expect("Could not set lua global functions.");

    State {
        engine: engine,
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
        module::add_module(source);
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

        // Ok clear the cached tables
        state.tables.borrow_mut().clear();

        // Ok now cler the GC.
        state.engine.gc_collect().unwrap();
    }

    fn start_thread() {
        // LUA does not need this.
    }

    fn stop_thread() {
        // LUA does not need this.
    }

    fn clear_state(call_gc: bool) {
        let state = get_lua_state();

        state.tables.borrow_mut();

        if call_gc {
            state.engine.gc_collect().unwrap();
        }
    }
    
    fn eval(code: &str) -> pxs_Var {
        let state = get_lua_state();
        let res = state.engine.load(code).call(());
        if res.is_err() {
            let msg = res.err().unwrap().to_string();
            return pxs_Var::new_string(msg);
        }
        let res: LuaValue = res.unwrap();

        from_lua(res).unwrap()   
    }
    
    fn compile(code: &str, global_scope: pxs_Var) -> pxs_Var {
        let state = get_lua_state();

        let globals = state.engine.globals();
        // Linking table between scope and globals
        let mt = state.engine.create_table().expect("Can not create table");
        mt.set("__index", globals).expect("Could not set __index");
        let scope_table: LuaTable;
        // Check that scope if a map
        if global_scope.is_map() {
            let res = into_lua(&state.engine, &global_scope).unwrap_or(LuaNil);
            if res.is_nil() {
                return pxs_Var::new_exception("Could not convert global scope into Lua table".to_string());
            }

            scope_table = res.as_table().unwrap().to_owned();
        } else if global_scope.is_null() {
            let res = state.engine.create_table();
            if res.is_err() {
                return pxs_Var::new_exception("Could not create new table in Lua".to_string());
            }
            scope_table = res.unwrap();
        } else {
            return pxs_Var::new_exception("Expected Map or Null for global scope.".to_string());
        }

        scope_table.set_metatable(Some(mt)).expect("Could not set meta table");

        // Compile code
        let chunk = state.engine.load(code);
        let chunk = chunk.set_environment(scope_table);
        let our_scope_table = chunk.environment().unwrap().to_owned();

        let res = chunk.into_function();
        if res.is_err() {
            return pxs_Var::new_exception("Could not convert chunk into Lua Function.".to_string());
        }
        let func = res.unwrap();

        // Now lets return our [CodeObject, Global Scope reference]
        let result = pxs_Var::new_list();
        let list = result.get_list().unwrap();

        let res = from_lua(mlua::Value::Function(func));
        if res.is_err() {
            return pxs_Var::new_exception("Could not convert Lua function into PXS.".to_string());
        }
        // Code Object
        list.add_item(res.unwrap());
        // Global Scope
        let res = from_lua(mlua::Value::Table(our_scope_table));
        if res.is_err() {
            return pxs_Var::new_exception("Could not convert Lua table into PXS.".to_string());
        }
        list.add_item(res.unwrap());
        
        result
    }
    
    fn exec_object(code: pxs_Var, local_scope: pxs_Var) -> pxs_Var {
        let state = get_lua_state();
        let lua = &state.engine;

        // We have to get the CodeObject and the Global Scope first
        let (code_object, global_scope) = {
            let list = code.get_list().unwrap();
            (list.get_item(1).unwrap(), list.get_item(2).unwrap())
        };

        // Now add local scope to global scope...
        let res = into_lua(lua, &global_scope);
        if res.is_err() {
            return pxs_Var::new_exception("Could not convert global_scope to Lua.".to_string());
        }
        // This is why it would be nice to remove mlua one day...
        let binding = res.unwrap();
        let global_table = binding.as_table();
        let global_table = global_table.unwrap();
        
        // Set local scope if not null
        if !local_scope.is_null() {
            let map = local_scope.get_map().unwrap();
            let keys = map.keys();
            for k in keys {
                // Key => LuaValue
                let res = into_lua(lua, k);
                if res.is_err() {
                    return pxs_Var::new_exception("Could not convert key into lua");
                }
                let lua_key = res.unwrap();
                // Value => LuaValue
                let value = map.get_item(k);
                if let Some(v) = value { 
                    let res = into_lua(lua, v);
                    if res.is_err() {
                        return pxs_Var::new_exception("Could not convert value into lua");
                    }

                    let lua_value = res.unwrap();
                    // Set in table
                    let ok = global_table.set(lua_key, lua_value);
                    if ok.is_err() {
                        return pxs_Var::new_exception("Could not set key=>value in Lua table.");
                    }
                }
            }
        }

        // Get the object as a function and call it
        let res = into_lua(lua, code_object);
        if res.is_err() {
            return pxs_Var::new_exception("Could not convert Code Object back into lua.");
        }
        // UGHHH Freaking borrow checker 
        let binding = res.unwrap();
        let func = binding.as_function();
        let func = func.unwrap();

        let res: LuaResult<LuaValue> = func.call(());
        if res.is_err() {
            return pxs_Var::new_exception(res.unwrap_err().to_string());
        }
        let result = res.unwrap();
        let res = from_lua(result);
        if res.is_err() {
            return pxs_Var::new_exception(res.unwrap_err().to_string());
        }

        // Now remove the local_state
        if !local_scope.is_null() {
            let map = local_scope.get_map().unwrap();
            let keys = map.keys();
            for k in keys {
                // Key => LuaValue
                let lua_key = get_lua_res!("Could not convert key into lua", into_lua, lua, k);
                // Remove the pair                
                let _ = global_table.set(lua_key, LuaNil);
            }
        }

        res.unwrap()
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
            // Just grab it from the ptr itself
            let table_ptr = var.value.object_val as *const LuaTable;
            (&*table_ptr).clone()
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

    fn get(var: &pxs_Var, key: &str) -> Result<pxs_Var, anyhow::Error> {
        // Get object from lua
        let table = unsafe {
            // Just grab it from the ptr itself
            let table_ptr = var.value.object_val as *const LuaTable;
            (&*table_ptr).clone()
        };

        let value: LuaValue = table.raw_get(key)?;
        
        from_lua(value)
    }
    
    fn set(var: &pxs_Var, key: &str, value: &pxs_Var) -> Result<pxs_Var, anyhow::Error> {
        // Get object from lua
        let table = unsafe {
            // Just grab it from the ptr itself
            let table_ptr = var.value.object_val as *const LuaTable;
            (&*table_ptr).clone()
        };

        let state = get_lua_state();
        let res = table.raw_set(key, into_lua(&state.engine, value)?);
        Ok(match res {
            Ok(_) => pxs_Var::new_bool(true),
            Err(_) => pxs_Var::new_bool(false),
        })
    }
    
    fn get_from_name(name: &str) -> Result<pxs_Var, anyhow::Error> {
        let state = get_lua_state();

        let res: LuaValue = state.engine.globals().get(name)?;
        from_lua(res)
    }
}
