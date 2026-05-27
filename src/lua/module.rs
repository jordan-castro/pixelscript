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
    lua::{State, func::internal_add_callback, into_lua},
    shared::module::pxs_Module,
};
use anyhow::Result;
use mlua::prelude::*;

/// Create the module table.
fn create_module(state: &mut State, module: &pxs_Module) -> Result<LuaTable> {
    let module_table = state.engine.create_table()?;

    // Add variables
    for variable in module.variables.iter() {
        module_table
            .set(
            variable.name.to_owned(),
            into_lua(state, &variable.var)?
            )?;
    }

    // Add callbacks
    for callback in module.callbacks.iter() {
        // Create lua function
        let lua_function = internal_add_callback(&state.engine, callback.idx);
        module_table
            .set(callback.name.as_str(), lua_function?)?;
    }

    Ok(module_table)
}

/// Add a module to Lua!
pub(super) fn add_module(state: &mut State, module: Arc<pxs_Module>) -> Result<()> {
    let module_for_lua = Arc::clone(&module);

    // Let's create a table
    let package: LuaTable = state
        .engine
        .globals()
        .get("package")?;
    let preload: LuaTable = package
        .get("preload")?;

    // Add internal modules.
    for child in module.modules.iter() {
        let child_module = child.clone();
        add_module(state, Arc::clone(&child_module))?;
    }

    let module_table_result = create_module(state, &module_for_lua);
    if module_table_result.is_err() {
        return Err(LuaError::RuntimeError(module_table_result.unwrap_err().to_string()).into());
    }
    let module_table = module_table_result.unwrap();
    // create the loader function for require()
    let loader = state
        .engine
        .create_function(move |_, _: ()| {
            Ok(module_table.to_owned())
            // let module_table = with_lua_state!(state => {create_module(&mut state, &module_for_lua)}) ;
            // if module_table.is_err() {
                // Err(LuaError::RuntimeError(module_table.unwrap_err().to_string()))
            // } else {
                // Return module
                // Ok(module_table.unwrap())
            // }
        })?;

    // Pre-load it
    preload
        .set(module.name.clone(), loader)?;

    Ok(())
}
