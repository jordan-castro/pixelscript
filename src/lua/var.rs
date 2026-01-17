// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{ffi::c_void, sync::Arc};

// use mlua::{IntoLua, Lua};
use mlua::prelude::*;

// Pure Rust goes here
use crate::{
    lua::object::create_object, shared::{
        object::get_object, var::{Var, VarType}
    }
};

/// Convert a Lua value to a Var.
pub(super) fn from_lua(value: LuaValue) -> Result<Var, anyhow::Error> {
    match value {
        LuaValue::Boolean(b) => Ok(Var::new_bool(b)),
        LuaValue::Integer(i) => Ok(Var::new_i64(i)),
        LuaValue::Number(n) => Ok(Var::new_f64(n)),
        LuaValue::String(s) => Ok(Var::new_string(s.to_string_lossy())),
        LuaValue::Table(t) => {
            let obj = Box::into_raw(Box::new(t));
            Ok(Var::new_object(obj as *mut c_void))
        },
        _ => {
            Ok(Var::new_null())
        }
        // LuaValue::LightUserData(light_user_data) => todo!(),
        // LuaValue::Table(table) => todo!(),
        // LuaValue::Function(function) => todo!(),
        // LuaValue::Thread(thread) => todo!(),
        // LuaValue::UserData(any_user_data) => todo!(),
        // LuaValue::Error(error) => todo!(),
        // LuaValue::Other(value_ref) => todo!(),
        // LuaNil => todo!(),
    }
}

/// Convert a Var into a LuaValue
pub(super) fn into_lua(lua: &Lua, var: &Var) -> LuaResult<LuaValue> {
    match var.tag {
            VarType::Int32 => Ok(mlua::Value::Integer(var.get_i32().unwrap() as i64)),
            VarType::Int64 => Ok(mlua::Value::Integer(var.get_i64().unwrap())),
            VarType::UInt32 => Ok(mlua::Value::Integer(var.get_u32().unwrap() as i64)),
            VarType::UInt64 => Ok(mlua::Value::Integer(var.get_u64().unwrap() as i64)),
            VarType::String => {
                let contents = var.get_string().unwrap();
                let lua_str = lua.create_string(contents).expect("test");

                Ok(mlua::Value::String(lua_str))
            }
            VarType::Bool => Ok(mlua::Value::Boolean(var.get_bool().unwrap())),
            VarType::Float32 => Ok(mlua::Value::Number(var.get_f32().unwrap() as f64)),
            VarType::Float64 => Ok(mlua::Value::Number(var.get_f64().unwrap())),
            VarType::Null => Ok(mlua::Value::Nil),
            VarType::Object => {
                unsafe {
                    // This MUST BE A TABLE!
                    let table_ptr = var.value.object_val as *const LuaTable;
                    if table_ptr.is_null() {
                        return Err(mlua::Error::RuntimeError(
                            "Null pointer in Object".to_string(),
                        ));
                    }

                    // Clone
                    let lua_table = (&*table_ptr).clone();

                    // WooHoo we are back into lua
                    Ok(mlua::Value::Table(lua_table))
                }
            }
            VarType::HostObject => {
                unsafe {
                    let idx = var.value.host_object_val;
                    // let object_lookup = get_object_lookup();
                    let pixel_object = get_object(idx).unwrap();
                    let lang_ptr_is_null = pixel_object.lang_ptr.lock().unwrap().is_null();
                    if lang_ptr_is_null {
                        // Create the table for the first time and mutate the pixel object.
                        let table = create_object(lua, idx, Arc::clone(&pixel_object));
                        // Add table ptr
                        let table_ptr = Box::into_raw(Box::new(table));
                        pixel_object.update_lang_ptr(table_ptr as *mut c_void);
                    }

                    // Get PTR again.
                    let lang_ptr = pixel_object.lang_ptr.lock().unwrap();
                    // Get as table.
                    let table_ptr = *lang_ptr as *const LuaTable;
                    // Return table
                    let table = (&*table_ptr).clone();
                    Ok(mlua::Value::Table(table))
                }
            }
        }
}

// /// Add a variable by name to __main__ in lua.
// pub fn add_variable(context: &Lua, name: &str, variable: &Var) {
//     context
//         .globals()
//         .set(
//             name,
//                 into_lua(context, variable)
//                 .expect("Could not unwrap LUA vl from Var."),
//         )
//         .expect("Could not add variable to Lua global context.");
//     // Listo!
// }
