use std::sync::Arc;

use crate::{lua::func::internal_add_callback, shared::object::PixelObject};
use mlua::prelude::*;

pub(super) fn create_object(lua: &Lua, idx: i32, source: Arc<PixelObject>) -> LuaTable {
    let table = lua.create_table().expect("Could not create table.");

    for callback in source.callbacks.iter() {
        // Get internals
        let func = callback.func.func;
        let opaque = callback.func.opaque;

        let lua_function = internal_add_callback(lua, func, opaque, Some(idx));
        table.set(callback.name.as_str(), lua_function).expect("Could not set callback to object");
    }

    table
}
