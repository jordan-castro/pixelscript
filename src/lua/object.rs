// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::sync::Arc;

use crate::{
    lua::{from_lua, get_metatable, into_lua, store_metatable},
    shared::{func::call_function, object::{ObjectFlags, pxs_PixelObject}, pxs_Runtime, var::pxs_Var},
};
use anyhow::{Result, anyhow};
use mlua::prelude::*;

fn create_object_callback(lua: &Lua, fn_idx: i32, flags: u8) -> Result<LuaFunction> {
    let func = lua.create_function(
        move |lua, (internal_obj, args): (LuaTable, LuaMultiValue)| -> Result<LuaValue, LuaError> {
            let mut argv = vec![];

            // Add runtime
            argv.push(pxs_Var::new_i64(pxs_Runtime::pxs_Lua as i64));

            // Check whether to pass id or not
            if flags & (ObjectFlags::UsesId as u8) == 1 {
                // Get obj id
                let obj_id: i64 = internal_obj
                    .get("_pxs_ptr")?;

                // Add object id
                argv.push(pxs_Var::new_i64(obj_id));
            } else {
                let obj_value = from_lua(internal_obj.to_value());
                if obj_value.is_err() {
                    return Err(LuaError::RuntimeError(obj_value.unwrap_err().to_string()));
                }
                argv.push(
                    obj_value.unwrap()
                );
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
            unsafe {
                let res = call_function(fn_idx, argv);
                // Convert into lua
                let lua_val = into_lua(lua, &res);
                lua_val
            }
        },
    );

    if func.is_err() {
        Err(anyhow!("{:#}", func.unwrap_err()))
    } else {
        Ok(func.unwrap())
    }
}

pub(super) fn create_object(lua: &Lua, idx: i32, source: Arc<pxs_PixelObject>) -> Result<LuaTable> {
    let table = lua.create_table()?;
    table.set("_pxs_ptr", LuaValue::Integer(idx as i64))?;

    let metatable = get_metatable(&source.type_name);

    // let mut state = get_state();
    let metatable = if let Some(mt) = metatable {
        mt.clone()
    } else {
        // Create new metatable
        let mt = lua.create_table()?;
        // Add methods
        for method in source.callbacks.iter() {
            let func = create_object_callback(lua, method.cbk.idx, method.flags)?;
            mt.set(method.cbk.name.clone(), func)?;
        }

        mt.set("__index", mt.clone())?;
        // save it
        store_metatable(&source.type_name, mt.clone());
        mt
    };

    table.set_metatable(Some(metatable))?;
    Ok(table)
}
