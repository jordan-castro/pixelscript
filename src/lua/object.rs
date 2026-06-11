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
    borrow_string, lua::{
        State, engine::Engine, func::lua_object_bridge, lua_error, lua
    }, shared::{
        PXS_PTR_NAME,
        object::{ObjectFlags, pxs_PixelObject}, utils::create_private_name,
    }
};

/// __index
unsafe extern "C" fn lua_index(L: *mut lua::lua_State) -> core::ffi::c_int {
    let table = 1;
    let key = 2;

    let function_name = unsafe {
        borrow_string!(lua::lua_tolstring(L, key, core::ptr::null_mut())).to_string()
    };
    // Possible private name.
    let private_name = create_private_name(&function_name);

    let mut engine = Engine::without_alloc(L);

    // Get the meta table because that is what has the values
    engine.get_meta(table); // 1 (3)
    let mt = engine.get_top();

    // Check for non private name first
    engine.push_string(&function_name); // 2 (4)
    engine.raw_get(mt); // 2 (4)

    if engine.get_type(engine.get_top()) != lua::LUA_TNIL as i32 {
        // We have our result!
        engine.remove(3); // remove MT
        return 1;
    }

    // We may have a property
    engine.pop(1); // 1 (3) Now only MT is on top.

    // Check private name
    engine.push_string(&private_name); // 2 (4)
    engine.raw_get(mt); // 2 (4)

    if engine.get_type(engine.get_top()) != lua::LUA_TFUNCTION as i32 {
        // Who knows what this is, just return it
        engine.remove(3); // remove MT
        return 1;
    }

    // We have our function on top!
    // Lets push the table
    engine.push_value(table); // 3 (5)
    let res = engine.call(1, 1); // if successfull 3 (5)
    if res.is_err() {
        engine.pop(1); // MT 
        return lua_error(L, &res.unwrap_err().to_string());
    }

    // We got a value!
    // Just need to return it
    // I need to remove MT only
    engine.remove(3);
    // Done
    1
}

/// __newindex
unsafe extern "C" fn lua_newindex(L: *mut lua::lua_State) -> core::ffi::c_int {
    let table = 1;
    let key = 2;
    let value = 3;

    let mut engine = Engine::new(L);

    // Property 
    let property_name = engine.to_string(key);
    let private_name = create_private_name(&property_name);

    // Get MT
    engine.get_meta(table);
    let mt = engine.get_top();

    // Check if MT has private name
    engine.push_string(&private_name);
    engine.raw_get(mt);

    if engine.get_type(engine.get_top()) != lua::LUA_TFUNCTION as i32 {
        // Set it like you would normally
        engine.push_value(key);
        engine.push_value(value);
        engine.set_table(table);
    } else {
        // This is a function!
        // Push table, value
        engine.push_value(table);
        engine.push_value(value);
        let res = engine.call(2, 1); // this should always be nil
        if res.is_err() {
            return lua_error(L, &res.unwrap_err().to_string());
        }
    }

    drop(engine);
    0
}

/// Create a new lua table and push it to stack. It's position on stack is returned.
pub(super) fn create_object(
    state: *mut State,
    idx: i32,
    source: Arc<pxs_PixelObject>,
) {
    let mut engine = Engine::from_state(state);

    let callback_count = source.callbacks.len();
    // Create the table off the `engine` tracked stack.
    unsafe {
        lua::lua_createtable((*state).engine, 0, callback_count as i32);
    }
    let table = engine.get_top();

    // Set the "_pxs_ptr"
    engine.push_string(PXS_PTR_NAME);
    engine.push_integer(idx);
    engine.set_table(table);

    // Create a new meta table
    let created = engine.new_meta(&source.type_name);
    if created == 0 {
        // Alaready exists
        engine.set_meta(table);
    }

    // Create a new meta table.
    let mt = engine.get_top();

    // Add callbacks
    for method in source.callbacks.iter() {
        // The method name changes if a prop _pxs{name}_.
        let method_name = if method.flags & ObjectFlags::IsProp as u8 != 0 {
            create_private_name(&method.cbk.name)
        } else {
            method.cbk.name.clone()
        };

        // Setup the function up values
        engine.push_integer(method.cbk.idx);
        engine.push_integer(method.flags as i32);
        engine.push_function(lua_object_bridge, 2);

        // Add to metatable
        engine.set_field(mt, &method_name);
    }

    // Bind __index
    engine.push_string("__index");
    engine.push_function(lua_index, 0);
    engine.raw_set(mt);

    // Bind __newindex
    engine.push_string("__newindex");
    engine.push_function(lua_newindex, 0);
    engine.raw_set(mt);

    // Just put it on the top
    engine.push_value(mt);

    // Assign MT to table
    engine.set_meta(table);
}
