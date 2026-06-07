// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//

// Pure Rust goes here
use crate::{
    borrow_string, create_raw_string, lua::{LuaReference, State, flua::{self, lua_createtable, lua_geti, lua_gettop, lua_rawseti, lua_settable}, get_lua_state, object::create_object}, pxs_error, shared::{
        PtrMagic, PxsRes, PxsResult, object::get_object, pxs_Runtime, utils::CStringSafe, var::{pxs_Var, pxs_VarObject, pxs_VarType}
    }
};

/// Convert a Lua value to a Var.
pub(super) fn from_lua(idx: i32) -> PxsResult {
    unsafe {
        let state = get_lua_state();
        // let lua_type = flua::lua_type((*state).engine, idx);

        #[allow(non_snake_case)]
        let L = (*state).engine;

        // Ensure `idx` is absolute
        let idx = if idx < 0 {
            lua_gettop(L) + idx + 1
        } else {
            idx
        };

        let lua_type = flua::lua_type(L, idx);

        if lua_type == flua::LUA_TNUMBER as i32 {
            if flua::lua_isinteger(L, idx) == 1 {
                Ok(pxs_Var::new_i64(flua::lua_tointegerx(L, idx, std::ptr::null_mut())))
            } else {
                Ok(pxs_Var::new_f64(flua::lua_tonumberx(L, idx, std::ptr::null_mut())))
            }
        }
        else if lua_type == flua::LUA_TBOOLEAN as i32 {
            let lua_bool = flua::lua_toboolean(L, idx);
            Ok(pxs_Var::new_bool(lua_bool == 1))
        }else if lua_type == flua::LUA_TSTRING as i32 {
            let lua_string = flua::lua_tolstring(L, idx, std::ptr::null_mut());
            let rust_string = borrow_string!(lua_string);
            Ok(pxs_Var::new_string(rust_string.to_string()))
        } else if lua_type == flua::LUA_TFUNCTION as i32 {
            // Register the lua value.
            let reference = LuaReference::new(idx);
            Ok(pxs_Var::new_function(reference.into_void(), None))
        } else if lua_type == flua::LUA_TTABLE as i32 {
            // Check length
            let t_length = flua::lua_rawlen(L, idx);

            if t_length == 0 {
                // Register table
                let reference = LuaReference::new(idx);
                Ok(pxs_Var::new_object(pxs_VarObject::new_lang_only(reference.into_void()), None))
            } else {
                // List dayo!
                let mut values = vec![];
                for i in 0..t_length {
                    let val = from_lua(lua_geti(L, idx, i as i64))?;
                    values.push(val);
                }
                // Convert into list
                Ok(pxs_Var::new_list_with(values))
            }
        } else if lua_type == flua::LUA_TNONE {
            pxs_error!("Reference does not exist.")
        } else {
            Ok(pxs_Var::new_null())
        }
    }
}


/// Push pxs_Var onto Lua stack.
pub(super) fn push_lua_stack(var: &pxs_Var) -> PxsRes<i32> {
    unsafe {
        let state = get_lua_state();
        #[allow(non_snake_case)]
        let L = (*state).engine;
        match var.tag {
            pxs_VarType::pxs_Int64 => flua::lua_pushinteger(L, var.get_i64()?),
            pxs_VarType::pxs_UInt64 => flua::lua_pushinteger(L, var.get_u64()? as i64),
            pxs_VarType::pxs_String => {
                let mut cstring = CStringSafe::new();
                let contents = var.get_string()?;
                flua::lua_pushstring(L, cstring.new_string(&contents));
            },
            pxs_VarType::pxs_Bool => {
                let b = if var.get_bool()? {
                    1
                }else {
                    0
                };
                flua::lua_pushboolean(L, b);
            },
            pxs_VarType::pxs_Float64 => flua::lua_pushnumber(L, var.get_f64()?),
            pxs_VarType::pxs_Null => flua::lua_pushnil(L),
            pxs_VarType::pxs_Object => {
                let object = var.get_object_ptr();
                let reference= LuaReference::from_borrow_void(object);
                reference.push();
            },
            pxs_VarType::pxs_HostObject => todo!(),
            pxs_VarType::pxs_List => {
                let items = &var.get_list().unwrap().vars;
                lua_createtable(L, items.len() as i32, 0);
                let table = lua_gettop(L);

                for item in items.iter() {
                    // Push to top of stack
                    let idx = push_lua_stack(item)?;
                    // Send it!
                    lua_rawseti(L, table, idx as i64);
                }
            },
            pxs_VarType::pxs_Function => {
                let val = var.get_function()?;
                let reference = LuaReference::from_borrow_void(val);
                reference.push();
            },
            pxs_VarType::pxs_Factory => {
                let factory = var.get_factory().unwrap();
                let res = factory.call(pxs_Runtime::pxs_Lua);
                push_lua_stack(&res);
            },
            pxs_VarType::pxs_Exception => {
                let msg = var.get_string()?;
                return pxs_error!("{msg}");
            },
            pxs_VarType::pxs_Map => {
                let map = var.get_map().unwrap();
                let keys = map.keys();
                lua_createtable(L, keys.len() as i32, 0);
                let table = lua_gettop(L);
                for k in keys {
                    push_lua_stack(k);
                    push_lua_stack(map.get_item(k).unwrap());
                    lua_settable(L, table);
                }
            },
        }

        Ok(lua_gettop(L))
    }
}

// /// Convert a Var into a LuaValue
// pub(super) fn into_lua(state: *mut State, var: &pxs_Var) -> LuaResult<LuaValue> {
//     match var.tag {
//         pxs_VarType::pxs_Int64 => Ok(mlua::Value::Integer(var.get_i64().unwrap())),
//         pxs_VarType::pxs_UInt64 => Ok(mlua::Value::Integer(var.get_u64().unwrap() as i64)),
//         pxs_VarType::pxs_String => {
//             let contents = var.get_string().unwrap().clone();
//             let lua_str = unsafe { (*state).engine.create_string(contents)? };

//             Ok(mlua::Value::String(lua_str))
//         }
//         pxs_VarType::pxs_Bool => Ok(mlua::Value::Boolean(var.get_bool().unwrap())),
//         pxs_VarType::pxs_Float64 => Ok(mlua::Value::Number(var.get_f64().unwrap())),
//         pxs_VarType::pxs_Null => Ok(mlua::Value::Nil),
//         pxs_VarType::pxs_Object => {
//             unsafe {
//                 // This MUST BE A TABLE!
//                 let table_ptr = var.get_object_ptr() as *const LuaTable;
//                 if table_ptr.is_null() {
//                     return Err(mlua::Error::RuntimeError(
//                         "Null pointer in Object".to_string(),
//                     ));
//                 }

//                 // Clone
//                 let lua_table = (&*table_ptr).clone();

//                 // WooHoo we are back into lua
//                 Ok(mlua::Value::Table(lua_table))
//             }
//         }
//         pxs_VarType::pxs_HostObject => {
//             unsafe {
//                 let idx = var.value.host_object_val;
//                 // let object_lookup = get_object_lookup();
//                 let pixel_object = get_object(idx).unwrap();
//                 let lang_ptr_is_null = pixel_object.lang_ptr.lock().unwrap().is_null();
//                 if lang_ptr_is_null {
//                     // Create the table for the first time and mutate the pixel object.
//                     let table = create_object(state, idx, Arc::clone(&pixel_object));
//                     // Add table ptr
//                     let table_ptr = Box::into_raw(Box::new(table));
//                     pixel_object.update_lang_ptr(table_ptr as *mut c_void);
//                     // pixel_object.update_pxs_free_method(free_lua_table);
//                 }

//                 // Get PTR again.
//                 let lang_ptr = pixel_object.lang_ptr.lock().unwrap();
//                 // Get as table.
//                 let table_ptr = *lang_ptr as *const LuaTable;
//                 // Return table
//                 let table = (&*table_ptr).clone();
//                 Ok(mlua::Value::Table(table))
//             }
//         }
//         pxs_VarType::pxs_List => {
//             // Have to convert each item to a lua variable
//             let table = unsafe { (*state).engine.create_table()? };

//             // Loop through items and BORROW them
//             for item in var.get_list().unwrap().vars.iter() {
//                 // Add them to table
//                 let lua_val = into_lua(state, item)?;
//                 table.push(lua_val)?;
//             }

//             Ok(mlua::Value::Table(table))
//         }
//         pxs_VarType::pxs_Function => {
//             unsafe {
//                 // This has got to be a function
//                 let func_ptr = var.value.function_val as *const LuaFunction;
//                 if func_ptr.is_null() {
//                     return Err(mlua::Error::RuntimeError(
//                         "Null pointer in Function".to_string(),
//                     ));
//                 }

//                 // Clone the function
//                 let lua_function = (&*func_ptr).clone();
//                 // Do I need to clone here?
//                 // Shouldn't I just return the value? Not sure...
//                 Ok(mlua::Value::Function(lua_function))
//             }
//         }
//         pxs_VarType::pxs_Factory => {
//             // Call and return
//             let factory = var.get_factory().unwrap();
//             let res = factory.call(pxs_Runtime::pxs_Lua);
//             // convert into lua
//             into_lua(state, &res)
//         }
//         pxs_VarType::pxs_Exception => {
//             // Get msg and error it
//             let msg = var.get_string();
//             if msg.is_err() {
//                 Err(mlua::Error::RuntimeError(msg.unwrap_err().to_string()))
//             } else {
//                 Err(mlua::Error::RuntimeError(msg.unwrap()))
//             }
//         }
//         pxs_VarType::pxs_Map => {
//             // Key,Value pair table
//             let table = unsafe { (*state).engine.create_table()? };

//             // Get keys
//             let map = var.get_map().unwrap();
//             let keys = map.keys();

//             for k in keys {
//                 let item = map.get_item(k);
//                 if let Some(item) = item {
//                     let lua_key = into_lua(state, k)?;
//                     let lua_val = into_lua(state, item)?;
//                     // Save
//                     table.set(lua_key, lua_val)?;
//                 }
//             }

//             Ok(mlua::Value::Table(table))
//         }
//     }
// }