// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{collections::HashMap, sync::Arc};

use crate::{
    lua::{State, from_lua, get_metatable, into_lua, store_metatable},
    shared::{
        ffi::ThreadSafePointer, func::call_function, object::{ObjectCallback, ObjectFlags, pxs_PixelObject}, pxs_Runtime, var::pxs_Var
    },
};
use anyhow::{Result, anyhow};
use mlua::prelude::*;

fn create_object_callback(state: *mut State, fn_idx: i32, flags: u8) -> Result<LuaFunction> {
    let thread_safe_state = ThreadSafePointer::<State>::new(state);
    let func = unsafe { (*state).engine.create_function(
        move |_, (internal_obj, args): (LuaTable, LuaMultiValue)| -> Result<LuaValue, LuaError> {
            let mut argv = vec![];

            // Add runtime
            argv.push(pxs_Var::new_i64(pxs_Runtime::pxs_Lua as i64));

            // Check whether to pass id or not
            if flags & (ObjectFlags::UsesId as u8) != 0 {
                // Get obj id
                let obj_id: i64 = internal_obj.get("_pxs_ptr")?;

                // Add object id
                argv.push(pxs_Var::new_i64(obj_id));
            } else {
                let obj_value = from_lua(internal_obj.to_value());
                if obj_value.is_err() {
                    return Err(LuaError::RuntimeError(obj_value.unwrap_err().to_string()));
                }
                argv.push(obj_value.unwrap());
            }
            // Add args
            for arg in args {
                let lua_arg = from_lua(arg);
                if lua_arg.is_err() {
                    return Err(LuaError::RuntimeError(lua_arg.unwrap_err().to_string()));
                }
                argv.push(lua_arg.unwrap());
            }

            // Call
            let res = call_function(fn_idx, argv);
            // Convert into lua
            let lua_val = into_lua(thread_safe_state.get_ptr(), &res);
            lua_val
        },
    ) };

    if func.is_err() {
        Err(anyhow!("{:#}", func.unwrap_err()))
    } else {
        Ok(func.unwrap())
    }
}

/// The __index function
fn lua_index(source: &LuaTable, table: &LuaTable, key: String, callbacks: HashMap<String, ObjectCallback>) -> LuaResult<LuaValue> {
    // Oki dok, here we need to go through callback names
    let method = callbacks.get(&key);
    if let Some(method) = method {
        // Check if it is a property
        if method.flags & ObjectFlags::IsProp as u8 != 0 {
            let func_name = format!("__pxs_{}__", method.cbk.name);
            let func: LuaFunction = source.raw_get(func_name)?;
            let res: LuaValue = func.call((table,))?;
            return Ok(res);
        } else {
            let value: LuaValue = source.raw_get(key.clone())?;
            if !value.is_nil() {
                return Ok(value);
            }
        }
    }

    Ok(LuaNil)
}

/// The __newindex function
fn lua_newindex(source: &LuaTable, table: &LuaTable, key: String, value: &LuaValue, callbacks: HashMap<String, ObjectCallback>) -> LuaResult<LuaValue> {
    let method = callbacks.get(&key);
    if let Some(method) = method {
        // Check if it is a property
        if method.flags & ObjectFlags::IsProp as u8 != 0 {
            let func_name = format!("__pxs_{}__", method.cbk.name);
            let func: LuaFunction = source.raw_get(func_name)?;
            let res: LuaValue = func.call((table, value))?;
            return Ok(res);
        } else {
            let value: LuaValue = source.raw_get(key.clone())?;
            if !value.is_nil() {
                return Ok(value);
            }
        }
    }

    Ok(LuaNil)
}

pub(super) fn create_object(state: *mut State, idx: i32, source: Arc<pxs_PixelObject>) -> Result<LuaTable> {
    let table = unsafe { (*state).engine.create_table()? };
    table.set("_pxs_ptr", LuaValue::Integer(idx as i64))?;

    let metatable = get_metatable(state, &source.type_name);

    // let mut state = get_state();
    let metatable = if let Some(mt) = metatable {
        mt.clone()
    } else {
        // Create new metatable
        let mt = unsafe { (*state).engine.create_table()? };

        // HashMap of String -> ObjectCallbackc
        let mut map: HashMap<String, ObjectCallback> = HashMap::new();

        // Add methods
        for method in source.callbacks.iter() {
            map.insert(method.cbk.name.clone(), method.clone());
            let func = create_object_callback(state, method.cbk.idx, method.flags)?;

            // Name needs to be warped potentially.
            let name = if method.flags & ObjectFlags::IsProp as u8 != 0 {
                format!("__pxs_{}__", method.cbk.name)
            } else {
                method.cbk.name.clone()
            };
            mt.set(name.clone(), func)?;
        }

        // Our custom index function
        let index_source_clone = mt.clone();
        let newindex_source_clone = mt.clone();
        let index_callbacks_clone = map.clone();
        let newindex_callbacks_clone = map.clone();
        let custom_index = unsafe { (*state).engine.create_function(move |_lua, (table, key): (LuaTable, String) | {
            Ok(lua_index(&index_source_clone, &table, key, index_callbacks_clone.clone())?)
        })? };
        let custom_newindex = unsafe { (*state).engine.create_function(move |_lua, (table, key, value): (LuaTable, String, LuaValue) | {
            Ok(lua_newindex(&newindex_source_clone, &table, key, &value, newindex_callbacks_clone.clone())?)
        })? };

        mt.set("__index", custom_index)?;
        mt.set("__newindex", custom_newindex)?;
        // save it
        store_metatable(state, &source.type_name, mt.clone());
        mt
    };

    table.set_metatable(Some(metatable))?;
    Ok(table)
}