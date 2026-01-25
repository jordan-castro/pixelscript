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
    shared::{pxs_Runtime, func::call_function, object::pxs_PixelObject, var::pxs_Var},
};
use mlua::prelude::*;

fn create_object_callback(lua: &Lua, fn_idx: i32) -> LuaFunction {
    lua.create_function(
        move |lua, (internal_obj, args): (LuaTable, LuaMultiValue)| {
            let mut argv = vec![];

            // Add runtime
            argv.push(pxs_Var::new_i64(pxs_Runtime::pxs_Lua as i64));

            // Get obj id
            let obj_id: i64 = internal_obj
                .get("_id")
                .expect("Could not grab ID from Object.");

            // Add object id
            argv.push(pxs_Var::new_i64(obj_id));

            // Add args
            for arg in args {
                argv.push(from_lua(arg).expect("Could not convert Lua Value into Var."));
            }

            // Call
            unsafe {
                let res = call_function(fn_idx, argv);
                // Convert into lua
                let lua_val = into_lua(lua, &res);
                lua_val
            }
        },
    )
    .expect("Could not create function on object")
}

pub(super) fn create_object(lua: &Lua, idx: i32, source: Arc<pxs_PixelObject>) -> LuaTable {
    let table = lua.create_table().expect("Could not create table.");
    table
        .set("_id", LuaValue::Integer(idx as i64))
        .expect("Could not set _id on Lua Table.");
    // Check if the meta table exists already

    let metatable = get_metatable(&source.type_name);

    // let mut state = get_state();
    let metatable = if let Some(mt) = metatable {
        mt.clone()
    } else {
        // Create new metatable
        let mt = lua.create_table().expect("Could not create Metatable");
        // Add methods
        for method in source.callbacks.iter() {
            let func = create_object_callback(lua, method.idx);
            mt.set(method.name.clone(), func)
                .expect("Could not set method");
        }

        mt.set("__index", mt.clone())
            .expect("Could not set __index");
        // save it
        store_metatable(&source.type_name, mt.clone());
        mt
    };

    table
        .set_metatable(Some(metatable))
        .expect("Could not attach Metatable Lua.");
    table
}
