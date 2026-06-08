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
        State, func::lua_bridge, lua, lua_get_error, lua_pop, lua_upvalueindex, push_string, var::push_lua_stack
    },
    pxs_error,
    shared::{PxsRes, module::pxs_Module, utils::CStringSafe},
};

/// Load function
unsafe extern "C" fn module_loader(L: *mut lua::lua_State) -> core::ffi::c_int {
    unsafe {
        lua::lua_pushvalue(L, lua_upvalueindex(1));
        1
    }
}

/// Compile a Lua chunk of code
pub(super) fn compile_chunk(L: *mut lua::lua_State, code: &str, name: &str) -> PxsRes<i32> {
    let mut cstring = CStringSafe::new();
    unsafe {
        let res = lua::luaL_loadbufferx(
            L,
            cstring.new_string(code),
            code.len(),
            cstring.new_string(name),
            core::ptr::null_mut(),
        );
        if res != lua::LUA_OK as i32 {
            let lua_error = lua_get_error(L);
            return pxs_error!("{lua_error}");
        }

        Ok(lua::lua_gettop(L))
    }
}

/// Preload a lua source code as a module.
pub(super) fn preload_lua_module(L: *mut lua::lua_State, code: &str, name: &str) -> PxsRes<()> {
    let mut cstring = CStringSafe::new();
    unsafe {
        lua::lua_getglobal(L, cstring.new_string("package"));
        push_string(L, "preload");
        lua::lua_rawget(L, -2);
        let preload_idx = lua::lua_gettop(L);

        compile_chunk(L, code, name)?;
        let status = lua::lua_pcallk(L, 0, 1, 0, 0, None);

        if status != lua::LUA_OK as i32 {
            let lua_error = lua_get_error(L);
            lua_pop(L, 2);
            return pxs_error!("{lua_error}");
        }

        // Pass result into upvalue
        lua::lua_pushcclosure(L, Some(module_loader), 1);
        lua::lua_setfield(L, preload_idx, cstring.new_string(name));

        // Now we need to drop everything
        lua_pop(L, 2);

        Ok(())
    }
}

pub(super) fn add_module(state: *mut State, module: Arc<pxs_Module>) -> PxsRes<()> {
    let mut cstring = CStringSafe::new();
    unsafe {
        let L = (*state).engine;

        // Create the table
        lua::lua_createtable(
            L,
            0,
            (module.variables.len() + module.callbacks.len()) as i32,
        );
        let table = lua::lua_gettop(L);

        for var in module.variables.iter() {
            push_string(L, &var.name);
            push_lua_stack(&var.var);
            lua::lua_rawset(L, table);
        }

        for callback in module.callbacks.iter() {
            push_string(L, &callback.name);
            lua::lua_pushinteger(L, callback.idx as i64);
            lua::lua_pushcclosure(L, Some(lua_bridge), 1);
            lua::lua_rawset(L, table);
        }

        lua::lua_getglobal(L, cstring.new_string("package"));
        push_string(L, "preload");
        lua::lua_rawget(L, -2);
        let preload_idx = lua::lua_gettop(L);

        // Now we have preload module

        // Module stuff
        push_string(L, &module.name);
        lua::lua_pushvalue(L, table);
        lua::lua_pushcclosure(L, Some(module_loader), 1);
        lua::lua_rawset(L, preload_idx); // push to the preload table

        lua_pop(L, 3);

        for child in module.modules.iter() {
            let _ = add_module(state, Arc::clone(&child));
        }

        Ok(())
    }
}
