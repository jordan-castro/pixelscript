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
        State, engine::Engine, func::lua_object_bridge, lua, lua_error, lua_pop, lua_remove
    }, shared::{
        PXS_PTR_NAME,
        object::{ObjectFlags, pxs_PixelObject},
    }
};

/// __index
unsafe extern "C" fn lua_index(L: *mut lua::lua_State) -> core::ffi::c_int {
    println!("lua_index");
        let table = 1;
        let key = 2;

        let function_name = unsafe {
            borrow_string!(lua::lua_tolstring(L, key, core::ptr::null_mut())).to_string()
        };
        let function_name = format!("__pxs{function_name}__");

        let mut engine = Engine::without_alloc(L);

        // Get the meta table because that is what has the values
        engine.get_meta(table);
        let mt = engine.get_top();
        
        // Get list of key to boolean (is property)
        engine.get_field(mt, "_pxs_props");
        // Is a table
        let pxs_props_table = engine.get_top();
        
        // Check for key in this table to know what to do with it
        engine.push_value(key);
        engine.get_table(pxs_props_table);

        // Check if bool
        if !engine.to_boolean(-1) {
            // This is a regular thingy. So just get it and return it
            // Pop pxs_props_tble
            engine.pop(1);
            engine.push_value(key);
            engine.raw_get(mt);

            // Remove the MT. Since the engine wont handle it.
            engine.remove(3);

            return 1;
        }
        
        // Ok yes it is a property
        // Lets remove the property table. We dont need it anymore
        engine.pop(1);

        // Lets get to work
        engine.push_string(&function_name);
        engine.raw_get(mt);
        // We have the callback
        // setup args
        engine.push_value(table);
        // Call func with table.
        let res = engine.call(1, 1);
        if res.is_err() {
            return lua_error(L, &res.unwrap_err().to_string());
        }

        // Result is on stack.
        // Remove MT
        engine.remove(3);
    1
}

/// __newindex
unsafe extern "C" fn lua_newindex(L: *mut lua::lua_State) -> core::ffi::c_int {
    let table = 1;
    let key = 2;
    let value = 3;

    let mut engine = Engine::without_alloc(L);

    // Setup function name.
    let function_name = unsafe {
        borrow_string!(lua::lua_tolstring(L, key, core::ptr::null_mut())).to_string()
    };
    let function_name = format!("__pxs{function_name}__");

    // Get the MT
    engine.get_meta(table);
    let mt = engine.get_top();

    // Check 


    // unsafe {

    //     lua::lua_getmetatable(L, table);
    //     let mt = lua::lua_gettop(L);

    //     lua::lua_pushvalue(L, key);
    //     lua::lua_rawget(L, mt);

    //     // Check if result is a table (which means a property)
    //     let index_result = lua::lua_gettop(L);
    //     let lua_type = lua::lua_type(L, index_result);

    //     if lua_type == lua::LUA_TTABLE as i32 {
    //         lua::lua_rawgeti(L, -1, 1);
    //         lua::lua_pushvalue(L, table);
    //         lua::lua_pushvalue(L, value);
    //         let status = lua::lua_pcallk(L, 2, 1, 0, 0, None);

    //         if status != lua::LUA_OK as i32 {
    //             return lua::lua_error(L);
    //         }

    //         lua::lua_settop(L, 3);
    //     } else {
    //         // Pop the raw get
    //         lua_pop(L, 1);
    //         lua::lua_pushvalue(L, key);
    //         lua::lua_pushvalue(L, value);
    //         lua::lua_rawset(L, table);
    //         // Pop the MT.
    //         lua_pop(L, 1);
    //     }

    // }
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
        // All methods are internal tables
        // Properties get `is_prop` == true
        let property_table = engine.create_table(0, 1);
        if method.flags & ObjectFlags::IsProp as u8 != 0 {
            engine.push_boolean(true)
        } else {
            engine.push_boolean(false);
        }
        engine.raw_set_index(property_table, 1);

        // Setup function up values
        engine.push_integer(method.cbk.idx);
        engine.push_integer(method.flags as i32);
        engine.push_function(lua_object_bridge, 2);

        // Add to table
        engine.raw_set_index(property_table, 2);

        // Add the table to the MT.
        engine.set_field(mt, &method.cbk.name);
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
