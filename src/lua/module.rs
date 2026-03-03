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
    lua::{func::internal_add_callback, get_lua_state, into_lua},
    shared::{PtrMagic, module::pxs_Module, var::pxs_Var},
};
use mlua::prelude::*;

/// Create the module table.
fn create_module(context: &Lua, module: &pxs_Module) -> LuaTable {
    let module_table = context.create_table().expect("Could not create LUA table.");

    // let mut variables = vec![];
    // variables.extend(module.variables.iter().cloned());
    // for f in module.factories.iter() {
    //     variables.push(ModuleVariable::new(f.name.clone(), unsafe {f.get_result()}));
    // }

    // Add variables
    for variable in module.variables.iter() {
        let var = unsafe {pxs_Var::from_borrow(variable.var) };
        module_table
            .set(
                variable.name.to_owned(),
                    into_lua(context, var)
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

    module_table
}

/// Add a module to Lua!
pub fn add_module(module: Arc<pxs_Module>) {
    // First get lua state
    let state = get_lua_state();

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

    // Add internal modules.
    for child in module.modules.iter() {
        let child_module = child.clone();
        add_module(Arc::clone(&child_module));
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
        .set(module.name.clone(), loader)
        .expect("Could not set Lua module loader.");

}
