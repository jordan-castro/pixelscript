// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// use mlua::{Integer, IntoLua, Lua, MultiValue, Value::Nil, Variadic};

use crate::{
    create_raw_string, free_raw_string,
    lua::{
        engine::Engine,
        from_lua, lua, lua_pop, lua_upvalueindex,
        object::{lua_index, lua_newindex},
        var::push_lua_stack,
    },
    own_string, pxs_error,
    shared::{
        PXS_PTR_NAME, PxsRes, func::call_function, object::ObjectFlags, pxs_Runtime,
    },
};

pub const LUA_OBJECT_BRIDGE_FUNCTION: i32 = 0;
pub const LUA_MODULE_BRIDGE_FUNCTION: i32 = 1;
pub const LUA_INDEX_BRIDGE_FUNCTION: i32 = 2;
pub const LUA_NEWINDEX_BRIDGE_FUNCTION: i32 = 3;

/// cbindgen:ignore
#[unsafe(no_mangle)]
unsafe extern "C" fn pxslua_free_ruststring(ptr: *mut core::ffi::c_char) {
    if !ptr.is_null() {
        let _ = own_string!(ptr);
    }
}

/// cbindgen:ignore
/// This is defined in libs/pxs_lua.h
/// The idea is that we let C handle the lua_errors
#[unsafe(no_mangle)]
unsafe extern "C" fn pxslua_rustbridge(
    L: *mut lua::lua_State,
    err_buff: *mut *mut core::ffi::c_char,
) -> core::ffi::c_int {
    let engine = Engine::without_alloc(L);
    // Get the function type up value
    let function_type = engine.get_upvalue(1);

    let res = if function_type == LUA_OBJECT_BRIDGE_FUNCTION {
        lua_object_bridge(L)
    } else if function_type == LUA_MODULE_BRIDGE_FUNCTION {
        lua_bridge(L)
    } else if function_type == LUA_INDEX_BRIDGE_FUNCTION {
        lua_index(L)
    } else if function_type == LUA_NEWINDEX_BRIDGE_FUNCTION {
        lua_newindex(L)
    } else {
        Ok(0)
    };

    match res {
        Ok(num) => num,
        Err(err) => {
            let raw_string = create_raw_string!(err);
            unsafe {
                *err_buff = raw_string;
            }
            -1
        }
    }
}

fn lua_object_bridge(L: *mut lua::lua_State) -> PxsRes<i32> {
    unsafe {
        let argc = lua::lua_gettop(L);
        let obj = 1; // object is always first (IF actually passed.)

        // Get fn idx
        let fn_idx = lua::lua_tointegerx(L, lua_upvalueindex(2), core::ptr::null_mut());
        let flags = lua::lua_tointegerx(L, lua_upvalueindex(3), core::ptr::null_mut());

        // Check that obj is a table
        if lua::lua_type(L, obj) != lua::LUA_TTABLE as i32 {
            return pxs_error!("self required.");
        }

        // Let's setup our callback
        let mut argv = vec![pxs_Runtime::pxs_Lua.into_var()];

        // Check flags
        if flags as u8 & (ObjectFlags::UsesId as u8) != 0 {
            // Get _pxs_ptr
            let ptr_string = create_raw_string!(PXS_PTR_NAME);
            lua::lua_pushstring(L, ptr_string);
            free_raw_string!(ptr_string);
            lua::lua_rawget(L, obj);
            argv.push(from_lua(-1)?);
            // pop
            lua_pop(L, 1);
        } else {
            lua::lua_pushvalue(L, 1);
            argv.push(from_lua(1).unwrap());
            // Pop pushed
            lua_pop(L, 1);
        }

        for i in 2..=argc {
            lua::lua_pushvalue(L, i);
            let var = from_lua(i);
            lua_pop(L, 1);
            if let Ok(var) = var {
                argv.push(var);
            } else {
                return pxs_error!("{}", var.unwrap_err().to_string());
            }
        }

        // Call the fuction
        let res = call_function(fn_idx as i32, argv);
        let success: Result<i32, String> = push_lua_stack(&res);
        if success.is_err() {
            return pxs_error!("{}", success.unwrap_err().to_string());
        }

        // Always return 1 dog
        Ok(1)
    }
}

/// The lua function bridge
fn lua_bridge(L: *mut lua::lua_State) -> PxsRes<i32> {
    unsafe {
        let argc = lua::lua_gettop(L);

        // Get fn idx
        let fn_idx = lua::lua_tointegerx(L, lua_upvalueindex(2), std::ptr::null_mut());

        // Now we have fn idx, lets set up our callback
        let mut argv = vec![pxs_Runtime::pxs_Lua.into_var()];

        // Num of args (skip first)
        for i in 1..=argc {
            lua::lua_pushvalue(L, i);
            let var = from_lua(-1);
            lua_pop(L, 1);
            if let Ok(var) = var {
                argv.push(var);
            } else {
                return pxs_error!("{}", var.unwrap_err().to_string());
            }
        }

        // Call callback
        let res = call_function(fn_idx as i32, argv);

        let success: Result<i32, String> = push_lua_stack(&res);
        if success.is_err() {
            return pxs_error!("{}", success.unwrap_err().to_string());
        }

        // Always return 1 as number of args returned.
        Ok(1)
    }
}
