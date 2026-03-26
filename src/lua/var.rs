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
    lua::object::create_object,
    shared::{
        object::get_object,
        pxs_Runtime,
        var::{pxs_Var, pxs_VarType},
    },
};

unsafe extern "C" fn free_lua_mem(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from(ptr);
}

// /// Lua Function for freeing memory
// unsafe extern "C" fn free_lua_object(ptr: *mut c_void) {
//     if ptr.is_null() {
//         return;
//     }
//     unsafe {
//         let table: LuaTable = *Box::from_raw(ptr as *mut LuaTable);
//         let pxs_ptr: LuaInteger = table.get("_pxs_ptr").unwrap_or(-1);
//         if pxs_ptr >= 0 {
//             // Free it
//             clear_object_from_lookup(pxs_ptr as i32);
//         }
//         // Table gets dropped here
//     }
// }

/// Convert a Lua value to a Var.
pub(super) fn from_lua(value: LuaValue) -> Result<pxs_Var, anyhow::Error> {
    match value {
        LuaValue::Boolean(b) => Ok(pxs_Var::new_bool(b)),
        LuaValue::Integer(i) => Ok(pxs_Var::new_i64(i)),
        LuaValue::Number(n) => Ok(pxs_Var::new_f64(n)),
        LuaValue::String(s) => Ok(pxs_Var::new_string(s.to_string_lossy())),
        LuaValue::Function(f) => {
            // Get as pointer
            let func = Box::into_raw(Box::new(f));
            Ok(pxs_Var::new_function(
                func as *mut c_void,
                Some(free_lua_mem),
            ))
        }
        LuaValue::Table(t) => {
            // Check if table is actually a list.
            let t_length = t.raw_len();

            if t_length == 0 {
                // Regular table
                let obj = Box::into_raw(Box::new(t));
                Ok(pxs_Var::new_object(obj as *mut c_void, Some(free_lua_mem)))
            } else {
                // It's a list.
                let mut values = vec![];
                for i in 0..t_length {
                    let val = from_lua(t.get(i + 1)?)?;
                    values.push(val);
                }
                // Convert into a pxs_Varlist
                let list_var = pxs_Var::new_list_with(values);
                // let values = t.
                Ok(list_var)
            }
        }
        LuaValue::Error(error) => {
            let msg = error.to_string();
            Ok(pxs_Var::new_exception(msg))
        }
        _ => Ok(pxs_Var::new_null()),
    }
}

/// Convert a Var into a LuaValue
pub(super) fn into_lua(lua: &Lua, var: &pxs_Var) -> LuaResult<LuaValue> {
    match var.tag {
        pxs_VarType::pxs_Int64 => Ok(mlua::Value::Integer(var.get_i64().unwrap())),
        pxs_VarType::pxs_UInt64 => Ok(mlua::Value::Integer(var.get_u64().unwrap() as i64)),
        pxs_VarType::pxs_String => {
            let contents = var.get_string().unwrap().clone();
            let lua_str = lua.create_string(contents)?;

            Ok(mlua::Value::String(lua_str))
        }
        pxs_VarType::pxs_Bool => Ok(mlua::Value::Boolean(var.get_bool().unwrap())),
        pxs_VarType::pxs_Float64 => Ok(mlua::Value::Number(var.get_f64().unwrap())),
        pxs_VarType::pxs_Null => Ok(mlua::Value::Nil),
        pxs_VarType::pxs_Object => {
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
        pxs_VarType::pxs_HostObject => {
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
        pxs_VarType::pxs_List => {
            // Have to convert each item to a lua variable
            let table = lua.create_table()?;

            // Loop through items and BORROW them
            for item in var.get_list().unwrap().vars.iter() {
                // Add them to table
                let lua_val = into_lua(lua, item)?;
                table.push(lua_val)?;
            }

            Ok(mlua::Value::Table(table))
        }
        pxs_VarType::pxs_Function => {
            unsafe {
                // This has got to be a function
                let func_ptr = var.value.function_val as *const LuaFunction;
                if func_ptr.is_null() {
                    return Err(mlua::Error::RuntimeError(
                        "Null pointer in Function".to_string(),
                    ));
                }

                // Clone the function
                let lua_function = (&*func_ptr).clone();
                // Do I need to clone here?
                // Shouldn't I just return the value? Not sure...
                Ok(mlua::Value::Function(lua_function))
            }
        }
        pxs_VarType::pxs_Factory => {
            // Call and return
            let factory = var.get_factory().unwrap();
            let res = factory.call(pxs_Runtime::pxs_Lua);
            // convert into lua
            into_lua(lua, &res)
        }
        pxs_VarType::pxs_Exception => {
            // Get msg and error it
            let msg = var.get_string();
            if msg.is_err() {
                Err(mlua::Error::RuntimeError(msg.unwrap_err().to_string()))
            } else {
                Err(mlua::Error::RuntimeError(msg.unwrap()))
            }
        }
        pxs_VarType::pxs_Map => {
            // Key,Value pair table
            let table = lua.create_table()?;

            // Get keys
            let map = var.get_map().unwrap();
            let keys = map.keys();

            for k in keys {
                let item = map.get_item(k);
                if let Some(item) = item {
                    let lua_key = into_lua(lua, k)?;
                    let lua_val = into_lua(lua, item)?;
                    // Save
                    table.set(lua_key, lua_val)?;
                }
            }

            Ok(mlua::Value::Table(table))
        }
    }
}
