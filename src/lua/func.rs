// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use anyhow::{Result, anyhow};
use mlua::prelude::*;
// use mlua::{Integer, IntoLua, Lua, MultiValue, Value::Nil, Variadic};

use crate::{lua::{from_lua, into_lua}, shared::{pxs_Runtime, func::call_function, var::pxs_Var}};

/// For internal use since modules also need to use the same logic for adding a Lua callback.
pub(super) fn internal_add_callback(lua: &Lua, fn_idx: i32) -> Result<LuaFunction> {
    let func = lua.create_function(move |lua, args: LuaMultiValue| -> Result<LuaValue, LuaError> {
        // Convert args -> argv for pixelmods
        let mut argv: Vec<pxs_Var> = vec![];

        // Pass in the runtime type
        argv.push(pxs_Var::new_i64(pxs_Runtime::pxs_Lua as i64));

        for arg in args {
            let lua_arg = from_lua(arg);
            if lua_arg.is_err() {
                return Err(LuaError::RuntimeError(lua_arg.unwrap_err().to_string()));
            }
            argv.push(lua_arg.unwrap());
        }

        unsafe {
            let res = call_function(fn_idx, argv);

            let lua_val = into_lua(lua, &res);
            lua_val
            // Memory will drop here, and Var will be automatically freed!
        }
    });
    if func.is_err() {
        Err(anyhow!(func.unwrap_err().to_string()))
    } else {
        Ok(func.unwrap())
    }
}
