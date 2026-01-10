use std::sync::Arc;

use crate::{lua::{get_metatable, store_metatable}, shared::{PixelScriptRuntime, func::call_function, object::PixelObject, var::Var}};
use mlua::prelude::*;

fn create_object_callback(lua: &Lua, fn_idx: i32) -> LuaFunction {
    lua.create_function(move |lua, (internal_obj, args): (LuaTable, LuaMultiValue)| {
        let mut argv = vec![];

        // Add runtime
        argv.push(Var::new_i64(PixelScriptRuntime::Lua as i64));

        // Get obj id
        let obj_id : i64 = internal_obj.get("_id").expect("Could not grab ID from Object.");

        // Add object id
        argv.push(Var::new_i64(obj_id));

        // Add args
        for arg in args {
            argv.push(Var::from_lua(arg, lua).expect("Could not convert Lua Value into Var."));
        }

        // Call
        unsafe {
            let res = call_function(fn_idx, argv);
            // Convert into lua
            let lua_val = res.into_lua(lua);
            lua_val
        }
    }).expect("Could not create function on object")
}

pub(super) fn create_object(lua: &Lua, idx: i32, source: Arc<PixelObject>) -> LuaTable {
    let table = lua.create_table().expect("Could not create table.");
    table.set("_id", LuaValue::Integer(idx as i64)).expect("Could not set _id on Lua Table.");
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
            mt.set(method.name.clone(), func).expect("Could not set method");
        }

        mt.set("__index", mt.clone()).expect("Could not set __index");
        // save it
        store_metatable(&source.type_name, mt.clone());
        mt
    };

    table.set_metatable(Some(metatable)).expect("Could not attach Metatable Lua.");
    table
}
