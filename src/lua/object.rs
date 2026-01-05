use std::sync::Arc;

use crate::{lua::func::internal_add_callback, shared::{func::get_function_lookup, object::PixelObject}};
use mlua::prelude::*;

pub(super) fn create_object(lua: &Lua, idx: i32, source: Arc<PixelObject>) -> LuaTable {
    let table = lua.create_table().expect("Could not create table.");

    // For methods within the creation of objects, the language needs to own the function since they are created at runtime
    let mut function_lookup = get_function_lookup();

    for callback in source.callbacks.iter() {
        // Get internals
        let func = callback.func.func;
        let opaque = callback.func.opaque;

        let fn_idx = function_lookup.add_function(func, opaque);

        let lua_function = internal_add_callback(lua, fn_idx, Some(idx));
        table.set(callback.name.as_str(), lua_function).expect("Could not set callback to object");
    }

    table
}
