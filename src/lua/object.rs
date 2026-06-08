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
    lua::{
        State, func::lua_object_bridge, lua, lua_pop, lua_remove, push_string
    }, shared::{
        PXS_PTR_NAME,
        object::{ObjectFlags, pxs_PixelObject},
        utils::CStringSafe,
    }
};

/// __index
unsafe extern "C" fn lua_index(L: *mut lua::lua_State) -> core::ffi::c_int {
    unsafe {
        let table = 1;
        let key = 2;

        lua::lua_getmetatable(L, table);
        let mt = lua::lua_gettop(L);

        lua::lua_pushvalue(L, key);
        lua::lua_rawget(L, mt);

        // Check if result is a table
        let index_result = lua::lua_gettop(L);
        let lua_type = lua::lua_type(L, index_result);

        if lua_type == lua::LUA_TTABLE as i32 {
            // This is a property. Check for 1, call, return value
            lua::lua_rawgeti(L, index_result, 1 as i64);
            let is_function = lua::lua_type(L, -1) == lua::LUA_TFUNCTION as i32;
            if !is_function {
                lua::lua_settop(L, 2);
                lua::lua_pushnil(L);
                return 1;
            }

            // Call the function
            lua::lua_pushvalue(L, table);
            let status = lua::lua_pcallk(L, 1, 1, 0, 0, None);
            if status != lua::LUA_OK as i32 {
                return lua::lua_error(L);
            }

            lua::lua_settop(L, 5);
            lua_remove(L, 4);
            lua_remove(L, 3);
        }
        // Not a table so it's just a regular function
        // so... just return it
    }
    1
}

/// __newindex
unsafe extern "C" fn lua_newindex(L: *mut lua::lua_State) -> core::ffi::c_int {
    unsafe {
        let table = 1;
        let key = 2;
        let value = 3;

        lua::lua_getmetatable(L, table);
        let mt = lua::lua_gettop(L);

        lua::lua_pushvalue(L, key);
        lua::lua_rawget(L, mt);

        // Check if result is a table (which means a property)
        let index_result = lua::lua_gettop(L);
        let lua_type = lua::lua_type(L, index_result);

        if lua_type == lua::LUA_TTABLE as i32 {
            lua::lua_rawgeti(L, -1, 1);
            lua::lua_pushvalue(L, table);
            lua::lua_pushvalue(L, value);
            let status = lua::lua_pcallk(L, 2, 1, 0, 0, None);

            if status != lua::LUA_OK as i32 {
                return lua::lua_error(L);
            }

            lua::lua_settop(L, 3);
        } else {
            // Pop the raw get
            lua_pop(L, 1);
            lua::lua_pushvalue(L, key);
            lua::lua_pushvalue(L, value);
            lua::lua_rawset(L, table);
            // Pop the MT.
            lua_pop(L, 1);
        }
        
    }
    0
}

/// Create a new lua table and push it to stack. It's position on stack is returned.
pub(super) fn create_object(
    state: *mut State,
    idx: i32,
    source: Arc<pxs_PixelObject>,
) -> i32 {
    unsafe {
        let mut cstring = CStringSafe::new();
        let L = (*state).engine;

        let callback_count = source.callbacks.len();
        lua::lua_createtable(L, 0, callback_count as i32);
        let table = lua::lua_gettop(L);

        // Set "_pxs_ptr"
        push_string(L, PXS_PTR_NAME);
        lua::lua_pushinteger(L, idx as i64);
        lua::lua_settable(L, table);

        // Create a new meta table.
        let created = lua::luaL_newmetatable(L, cstring.new_string(&source.type_name));
        if created == 0 {
            // already exists
            lua::lua_setmetatable(L, table);
            return table;
        }

        // Define it
        let mt = lua::lua_gettop(L);

        for method in source.callbacks.iter() {
            // Properties are `table[function]`
            let is_prop = if method.flags & ObjectFlags::IsProp as u8 != 0 {
                true
            } else {
                false
            };

            if is_prop {
                // New table
                lua::lua_createtable(L, 1, 0);
            }

            // Up values 
            // idx
            lua::lua_pushinteger(L, method.cbk.idx as i64);
            // flags
            lua::lua_pushinteger(L, method.flags as i64);
            // Create closure
            lua::lua_pushcclosure(L, Some(lua_object_bridge), 2);

            if is_prop {
                // Add to prop table
                lua::lua_rawseti(L, -2, 1);
            }

            let field_name = cstring.new_string(&method.cbk.name);
            lua::lua_setfield(L, mt, field_name);
        }

        // Bind __index
        push_string(L, "__index");
        lua::lua_pushcclosure(L, Some(lua_index), 0);
        lua::lua_rawset(L, mt);

        // Bind __newindex
        push_string(L, "__newindex");
        lua::lua_pushcclosure(L, Some(lua_newindex), 0);
        lua::lua_rawset(L, mt);

        // Assign mt to table
        lua::lua_setmetatable(L, table);

        table
    }
}
