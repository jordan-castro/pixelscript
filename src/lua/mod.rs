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

use anyhow::{Result, anyhow};
use mlua::prelude::*;
use std::collections::HashMap;

use crate::{
    lua::var::{from_lua, into_lua}, shared::{PixelScript, PtrMagic, ffi::ThreadLanguageState, read_file, var::{ObjectMethods, pxs_Var, pxs_VarMap}}, with_feature
};

thread_local! {
    static LUASTATE: ThreadLanguageState<State> = init_state();
}

/// This is the Lua state. Each language gets it's own private state
struct State {
    /// The lua engine.
    engine: Lua,
    /// Cached Tables
    tables: HashMap<String, LuaTable>,
}

impl PtrMagic for State {}

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
fn init_state() -> ThreadLanguageState<State> {
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
        let _ = preload_lua_module(&engine, include_str!("../../libs/dkjson.lua"), "__dkjson__");
        // Load in the pxs_json module
        let _ = preload_lua_module(&engine, include_str!("../../core/lua/pxs_json.lua"), "pxs_json");
        // Import it globally
        lua_globals.push_str("\npxs_json = require('pxs_json')\n");
    });
    let _ = engine.load(lua_globals).set_name("<lua_globals>").exec();

    let s = State {
        engine: engine,
        tables: HashMap::new(),
    };

    ThreadLanguageState::<State>::new(s.into_raw())
}

/// Get the state of LUA.
fn get_lua_state() -> *mut State {
    LUASTATE.with(|mutex| {
        mutex.get_ptr()
    })
}

/// Get a cached metatable from lua.
pub(self) fn get_metatable(state: *mut State, name: &str) -> Option<LuaTable> {
    unsafe { (*state).tables.get(name).cloned() }
}

/// Cahce a metatable.
pub(self) fn store_metatable(state: *mut State, name: &str, table: LuaTable) {
    unsafe {
        (*state).tables.insert(name.to_string(), table);
    }
}

/// Execute some orbituary lua code.
/// Returns a String. Empty means no error happened and was successful!
pub(self) fn execute(state: *mut State, code: &str, file_name: &str) -> String {
    let res = unsafe {
        (*state).engine.load(code).set_name(file_name).exec()
    };
    if res.is_err() {
        return res.unwrap_err().to_string();
    }

    String::from("")
}

/// Custom moduile loader function
fn setup_module_loader(lua: &Lua) -> Result<()> {
    // Get package.searchers
    let package: LuaTable = lua
        .globals()
        .get("package")?;
    let searchers: LuaTable = package
        .get("searchers")?;

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
        })?;

    // Set our loader in searchers list
    let len = searchers
        .len()?;
    searchers
        .set(len + 1, loader)?;

    Ok(())
}

/// Add variables to a Table from a Map
fn add_variables_to_table(state: *mut State, table: &LuaTable, map: &pxs_VarMap) -> Result<()> {
    let keys = map.keys();
    for k in keys {
        // Convert to lua
        let lkey = into_lua(state, k)?;
        let value = map.get_item(k);
        if let Some(v) = value {
            // convert to lua
            let lval = into_lua(state, v)?;
            // Set in table
            table.set(lkey, lval)?;
        }
    }

    Ok(())
}

/// Remove variables from a Table.
fn remove_variables_from_table(state: *mut State, table: &LuaTable, map: &pxs_VarMap) -> Result<()> {
    let keys = map.keys();
    for k in keys {
        // Convert to lua
        let lkey = into_lua(state, k)?;
        table.set(lkey, LuaNil)?;
    }

    Ok(())
}

pub struct LuaScripting;

impl PixelScript for LuaScripting {
    fn add_module(source: std::sync::Arc<crate::shared::module::pxs_Module>) {
        let state = get_lua_state();
        let res = module::add_module(state, source);
        if res.is_err() {
            panic!("{:#?}", res);
        }
    }

    fn execute(code: &str, file_name: &str) -> Result<pxs_Var> {
        let state = get_lua_state();
        let err = execute(state, code, file_name);
        if err.is_empty() {
            Ok(pxs_Var::new_null())
        } else {
            Ok(pxs_Var::new_exception(err))
        }
    }

    fn start() {
        // Initalize the state
        let state = get_lua_state();
        let res = unsafe { setup_module_loader(&(*state).engine) };
        if res.is_err() {
            panic!("{:#?}", res);
        }
    }

    fn stop() {
        Self::clear_state(true);
    }

    fn start_thread() {
        Self::start();
    }

    fn stop_thread() {
        Self::stop();
    }

    fn clear_state(call_gc: bool) {
        let state = get_lua_state();

        unsafe {
            (*state).tables.clear();

            if call_gc {
                (*state).engine.gc_collect().unwrap();
            }
        }
    }
    
    fn eval(code: &str) -> Result<pxs_Var> {
        let state = get_lua_state();
        let res: LuaValue = unsafe { (*state).engine.load(code).set_name("<lua_eval>").call(())? };
        Ok(from_lua(res)?)   
    }
    
    fn compile(code: &str, global_scope: pxs_Var) -> Result<pxs_Var> {
        let state = get_lua_state();

        unsafe {
        let globals = (*state).engine.globals();
        // Linking table between scope and globals
        let mt = (*state).engine.create_table()?;
        mt.set("__index", globals)?;
        let scope_table: LuaTable;
        // Check that scope if a map
        if global_scope.is_map() {
            let res = into_lua(state, &global_scope)?;
            scope_table = res.as_table().unwrap().to_owned();
        } else if global_scope.is_null() {
            let res = (*state).engine.create_table()?;
            scope_table = res;
        } else {
            return Ok(pxs_Var::new_exception("Expected Map or Null for global scope."));
        }

        scope_table.set_metatable(Some(mt))?;

        // Compile code
        let chunk = (*state).engine.load(code);
        let chunk = chunk.set_environment(scope_table).set_name("<lua_code_block>");
        let our_scope_table = chunk.environment().unwrap().to_owned();

        let func = chunk.into_function()?;

        // Now lets return our [CodeObject, Global Scope reference]
        let result = pxs_Var::new_list();
        let list = result.get_list().unwrap();

        // Code Object
        list.add_item(from_lua(mlua::Value::Function(func))?);
        // Global Scope
        list.add_item(from_lua(mlua::Value::Table(our_scope_table))?);
        
        Ok(result)
    }
    }
    
    fn exec_object(code: pxs_Var, local_scope: pxs_Var) -> Result<pxs_Var> {
        let state = get_lua_state();

        // We have to get the CodeObject and the Global Scope first
        let (code_object, global_scope) = {
            let list = code.get_list().unwrap();
            (list.get_item(1).unwrap(), list.get_item(2).unwrap())
        };

        // Now add local scope to global scope...
        let binding = into_lua(state, global_scope)?;
        let potential_table: Option<&LuaTable> = binding.as_table();
        if potential_table.is_none() {
            return Ok(pxs_Var::new_exception("Globals is not a Table."));
        }
        let global_table: LuaTable = potential_table.unwrap().to_owned();
        
        // Set local scope if not null
        if !local_scope.is_null() {
            let map = local_scope.get_map().unwrap();
            add_variables_to_table(state, &global_table, map)?;
        }

        // Get the object as a function and call it
        let binding = into_lua(state, code_object)?;
        let potential_func: Option<&LuaFunction> = binding.as_function();
        if potential_func.is_none() {
            return Ok(pxs_Var::new_exception("Code Object is not a function."));
        }
        let func = potential_func.unwrap();
        let res: LuaValue = func.call(())?;
        let pxs_res = from_lua(res)?;

        // Now remove the local_state
        if !local_scope.is_null() {
            let map = local_scope.get_map().unwrap();
            remove_variables_from_table(state, &global_table, map)?;
        }

        Ok(pxs_res)
    }
    
    fn debug() -> String {
        let state = get_lua_state();
        let tables = unsafe { &(*state).tables };
        format!("{{tables: {:#?}}}", tables)
    }
    
    fn reset() {
        Self::clear_state(true);
    }
}

/// Convert args for ObjectMethods into LuaMutliValue
fn args_to_lua(args: &Vec<pxs_Var>) -> LuaMultiValue {
    let mut lua_args = vec![];
    let state = get_lua_state();
    for arg in args.iter() {
        let lua_arg = into_lua(state, arg);
        if lua_arg.is_err() {
            lua_args.push(into_lua(state, &pxs_Var::new_exception(lua_arg.unwrap_err().to_string())).unwrap_or(LuaNil));
        }
        lua_args.push(into_lua(state, arg).unwrap_or(LuaNil));
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
            let table_ptr = var.get_object_ptr() as *const LuaTable;
            (&*table_ptr).clone()
        };

        let lua_args = args_to_lua(&args.vars);
        let res = table
            .call_function(method, lua_args)?;

        let pixel_res = from_lua(res)?;

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

        let function: LuaFunction = unsafe { (*state) .engine.globals().get(method)? };
        let res: LuaValue = function
            .call(lua_args)?;

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
        let res: LuaValue = (unsafe { &*lua_function }).call(lua_args)?;

        // Convert into pxs
        from_lua(res)
    }

    fn get(var: &pxs_Var, key: &str) -> Result<pxs_Var, anyhow::Error> {
        // Get object from lua
        let table = unsafe {
            // Just grab it from the ptr itself
            let table_ptr = var.get_object_ptr() as *const LuaTable;
            (&*table_ptr).clone()
        };

        let value: LuaValue = table.raw_get(key)?;
        from_lua(value)
    }
    
    fn set(var: &pxs_Var, key: &str, value: &pxs_Var) -> Result<pxs_Var, anyhow::Error> {
        // Get object from lua
        let table = unsafe {
            // Just grab it from the ptr itself
            let table_ptr = var.get_object_ptr() as *const LuaTable;
            (&*table_ptr).clone()
        };

        let state = get_lua_state();
        let res = table.raw_set(key, into_lua(state, value)?);
        Ok(match res {
            Ok(_) => pxs_Var::new_bool(true),
            Err(_) => pxs_Var::new_bool(false),
        })
    }
    
    fn get_from_name(name: &str) -> Result<pxs_Var, anyhow::Error> {
        let state = get_lua_state();

        let res: LuaValue = unsafe { (*state).engine.globals().get(name)? };
        from_lua(res)
    }
}