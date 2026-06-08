// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// use mlua::{Integer, IntoLua, Lua, MultiValue, Value::Nil, Variadic};

use crate::{create_raw_string, free_raw_string, lua::{lua, from_lua, lua_error, lua_pop, lua_upvalueindex, var::push_lua_stack}, shared::{PXS_PTR_NAME, func::call_function, object::ObjectFlags, pxs_Runtime}};

pub(super) unsafe extern "C" fn lua_object_bridge(L: *mut lua::lua_State) -> core::ffi::c_int {
    unsafe {
        let argc = lua::lua_gettop(L);
        let obj = 1; // object is always first

        // Get fn idx
        let fn_idx = lua::lua_tointegerx(L, lua_upvalueindex(1), core::ptr::null_mut());
        let flags = lua::lua_tointegerx(L, lua_upvalueindex(2), core::ptr::null_mut());

        // Check that obj is a table
        if lua::lua_type(L, obj) != lua::LUA_TTABLE as i32 {
            return lua_error(L, "self required.");
        }

        // Let's setup our callback
        let mut argv = vec![
            pxs_Runtime::pxs_Lua.into_var()
        ];

        // Check flags
        if flags as u8 & (ObjectFlags::UsesId as u8) != 0 {
            // Get _pxs_ptr
            let ptr_string = create_raw_string!(PXS_PTR_NAME);
            lua::lua_pushstring(L, ptr_string);
            free_raw_string!(ptr_string);
            lua::lua_rawget(L, obj);
            argv.push(from_lua(-1).unwrap());
            // pop
            lua_pop(L, 1);
        } else {
            argv.push(from_lua(1).unwrap());
        }

        for i in 2..=argc {
            argv.push(from_lua(i).unwrap());
        }

        // Call the fuction
        let res = call_function(fn_idx as i32, argv);
        let success: Result<i32, String> = push_lua_stack(&res);
        if success.is_err() {
            return lua_error(L, &success.unwrap_err().to_string());
        }

        // Always return 1 dog
        1
    }
}

/// The lua function bridge
pub(super) unsafe extern "C" fn lua_bridge(L: *mut lua::lua_State) -> core::ffi::c_int {
    unsafe {
        let argc = lua::lua_gettop(L);

        // Get fn idx
        let fn_idx = lua::lua_tointegerx(L, lua_upvalueindex(1), std::ptr::null_mut());

        // Now we have fn idx, lets set up our callback
        let mut argv = vec![
            pxs_Runtime::pxs_Lua.into_var()
        ];

        // Num of args (skip first)
        for i in 1..=argc {
            let var = from_lua(i);
            if let Ok(var) = var {
                argv.push(var);
            } else {
                return lua_error(L, &var.unwrap_err().to_string());
            }
        }

        // Call callback
        let res = call_function(fn_idx as i32, argv);

        let success: Result<i32, String> = push_lua_stack(&res);
        if success.is_err() {
            return lua_error(L, &success.unwrap_err().to_string());
        }

        // Always return 1 as number of args returned.
        1
    }
}
