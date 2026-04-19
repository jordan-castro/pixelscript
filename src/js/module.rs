// // Copyright 2026 Jordan Castro <jordan@grupojvm.com>
// //
// // Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
// //
// // http://www.apache.org/licenses/LICENSE-2.0
// //
// // Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
// //
// use std::sync::Arc;

// use crate::{
//     lua::{func::internal_add_callback, get_lua_state, into_lua},
//     shared::{PtrMagic, module::pxs_Module, var::pxs_Var},
// };
// use anyhow::Result;
// use mlua::prelude::*;

// /// Create the module table.
// fn create_module(context: &Lua, module: &pxs_Module) -> Result<LuaTable> {
//     let module_table = context.create_table()?;

//     // Add variables
//     for variable in module.variables.iter() {
//         let var = unsafe {pxs_Var::from_borrow(variable.var) };
//         module_table
//             .set(
//             variable.name.to_owned(),
//             into_lua(context, var)?
//             )?;
//     }

//     // Add callbacks
//     for callback in module.callbacks.iter() {
//         // Create lua function
//         let lua_function = internal_add_callback(context, callback.idx);
//         module_table
//             .set(callback.name.as_str(), lua_function?)?;
//     }

//     Ok(module_table)
// }

use std::sync::Arc;

use anyhow::Result;
use rquickjs::{IntoJs, Module};

use crate::{js::get_js_state, shared::module::pxs_Module};

/// Add a module to JS!
pub(super) fn add_module(module: Arc<pxs_Module>) -> Result<()> {
    // JS state dude
    let state = get_js_state();
    state.modules.borrow_mut().insert(module.name.clone(), Arc::clone(&module));
    Ok(())
}