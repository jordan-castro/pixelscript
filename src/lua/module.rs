use std::sync::Arc;

use crate::{
    lua::{func::internal_add_callback, get_lua_state},
    shared::module::Module,
};
use mlua::prelude::*;

/// Create the module table.
fn create_module(context: &Lua, module: &Module) -> LuaTable {
    let module_table = context.create_table().expect("Could not create LUA table.");

    // Add variables
    for variable in module.variables.iter() {
        module_table
            .set(
                variable.name.to_owned(),
                variable
                    .var
                    .clone()
                    .into_lua(context)
                    .expect("Could not convert variable to Lua."),
            )
            .expect("Could not set variable to module.");
    }

    // Add callbacks
    for callback in module.callbacks.iter() {
        // Create lua function
        let lua_function = internal_add_callback(context, callback.idx);
        module_table
            .set(callback.name.as_str(), lua_function)
            .expect("Could not set callback to module");
    }

    // // Add internal modules
    // for inner_module in module.modules.iter() {
    //     // Create a module
    //     let inner_table = create_module(context, inner_module);
    //     // Add to this module
    //     module_table
    //         .set(inner_module.name.to_owned(), inner_table)
    //         .expect("Could not create inner module.");
    // }

    module_table
}

/// Add a module to Lua!
pub fn add_module(module: Arc<Module>, parent: Option<&str>) {
    // First get lua state
    let state = get_lua_state();

    let mod_name = match parent {
        Some(p) => format!("{p}.{}", module.name.clone()),
        None => module.name.clone(),
    };
    let module_for_lua = Arc::clone(&module);

    // Let's create a table
    let package: LuaTable = state
        .engine
        .globals()
        .get("package")
        .expect("Could not grab the Package table");
    let preload: LuaTable = package
        .get("preload")
        .expect("Could not grab the Preload table");

    for child in module.modules.iter() {
        let child_module = child.clone();
        add_module(Arc::new(child_module), Some(mod_name.as_str()));
    }

    // create the loader function for require()
    let loader = state
        .engine
        .create_function(move |lua, _: ()| {
            let module_table = create_module(lua, &module_for_lua);
            // Return module
            Ok(module_table)
        })
        .expect("Could not load LUA module.");

    // Pre-load it
    preload
        .set(mod_name, loader)
        .expect("Could not set Lua module loader.");

}
