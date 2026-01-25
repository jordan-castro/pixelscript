// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use mlua::prelude::*;
// use mlua::{Integer, IntoLua, Lua, MultiValue, Value::Nil, Variadic};

use crate::{lua::{from_lua, into_lua}, shared::{pxs_Runtime, func::call_function, var::pxs_Var}};

/// For internal use since modules also need to use the same logic for adding a Lua callback.
pub(super) fn internal_add_callback(lua: &Lua, fn_idx: i32) -> LuaFunction {
    lua.create_function(move |lua, args: LuaMultiValue| {
        // Convert args -> argv for pixelmods
        let mut argv: Vec<pxs_Var> = vec![];

        // Pass in the runtime type
        argv.push(pxs_Var::new_i64(pxs_Runtime::pxs_Lua as i64));

        // Objects are handled a little differently know. It's kinda repeated code but oh well.
        // // If a obj is passed
        // if let Some(obj) = obj {
        //     // Add the pointer.
        //     argv.push(Var::new_i64(obj as i64));
        // }

        for arg in args {
            argv.push(from_lua(arg).expect("Could not convert value into Var from Lua."));
        }        

        unsafe {
            let res = call_function(fn_idx, argv);

            let lua_val = into_lua(lua, &res);
            lua_val
            // Memory will drop here, and Var will be automatically freed!
        }
    }).expect("Could not create lua function")
}

// /// Add a callback to lua __main__ context.
// pub(super) fn add_callback(name: &str, fn_idx: i32) {
//     let state = get_lua_state();
//     let lua_func = internal_add_callback(&state.engine, fn_idx);
//     state.engine.globals().set(name, lua_func).expect("Could not add callback to Lua.");
// }