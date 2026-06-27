// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::sync::Arc;

use etffi::cstring::CStringSafe;

use crate::{
    lua::{
        State, engine::Engine, func::LUA_MODULE_BRIDGE_FUNCTION, lua, lua_get_error, lua_upvalueindex, LUA_OK
    },
    pxs_error,
    shared::{PxsRes, module::pxs_Module},
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
        if res != LUA_OK {
            let lua_error = lua_get_error(L);
            return pxs_error!("{lua_error}");
        }

        Ok(lua::lua_gettop(L))
    }
}

/// Preload a lua source code as a module.
pub(super) fn preload_lua_module(L: *mut lua::lua_State, code: &str, name: &str) -> PxsRes<()> {
    let mut engine = Engine::new(L);
    // Get package
    engine.get_global("package");
    engine.get_field(-1, "preload");
    let preload_idx = engine.get_top();

    // Compile code
    engine.compile_chunk(code, name)?;
    // run it
    engine.call(0, 1)?;

    // Pass result ito upvalue
    engine.push_function(module_loader, 1);
    engine.set_field(preload_idx, name);

    Ok(())
}

pub(super) fn add_module(state: *mut State, module: Arc<pxs_Module>) -> PxsRes<()> {
    let mut engine = Engine::from_state(state);

    // Create module table
    let table = engine.create_table(0, (module.variables.len() +module.callbacks.len()) as i32);

    // Add variables
    for var in module.variables.iter() {
        engine.push_string(&var.name);
        engine.push_pxs(&var.var)?;
        engine.raw_set(table);
    }

    // Add callbacks
    for cbk in module.callbacks.iter() {
        engine.push_string(&cbk.name);
        engine.push_integer(LUA_MODULE_BRIDGE_FUNCTION);
        engine.push_integer(cbk.idx); // add idx to upvalue.
        engine.push_function(lua::pxslua_callback, 2);
        engine.raw_set(table);
    }

    // Setup loader

    // Get preload
    engine.get_global("package");
    engine.push_string("preload");
    engine.raw_get(-2);
    let preload_idx = engine.get_top();

    // Now we actually have the preload module
    // module stuff
    engine.push_string(&module.name);
    engine.push_value(table); // add table to upvalues
    engine.push_function(module_loader, 1);
    engine.raw_set(preload_idx);

    // Drop the engine to clean stack.
    drop(engine);

    for child in module.modules.iter() {
        add_module(state, Arc::clone(&child))?;
    }

    Ok(())
}
