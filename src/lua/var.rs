// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//

use std::sync::Arc;

use etffi::{borrow_string, cstring::CStringSafe};

// Pure Rust goes here
use crate::{
    lua::{LUA_TBOOLEAN, LUA_TFUNCTION, LUA_TNONE, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE, LuaReference, get_lua_state, lua::{self, lua_createtable, lua_geti, lua_gettop, lua_rawseti, lua_settable}, lua_pop, object::create_object}, pxs_error, shared::{
        PxsRes, PxsResult, object::get_object, pxs_Opaque, pxs_Runtime, var::{pxs_Var, pxs_VarObject, pxs_VarType}
    }
};
use etffi::ptr_magic::PtrMagic;

/// Free lua memory
unsafe extern "C" fn free_lua_mem(ptr: pxs_Opaque) {
    let _ = LuaReference::from_raw(ptr as *mut LuaReference);
}

/// Convert a Lua value to a Var.
pub(super) fn from_lua(idx: i32) -> PxsResult {
    unsafe {
        let state = get_lua_state();

        #[allow(non_snake_case)]
        let L = (*state).engine;

        // Ensure `idx` is absolute
        let idx = if idx < 0 {
            lua_gettop(L) + idx + 1
        } else {
            idx
        };

        let lua_type = lua::lua_type(L, idx);

        if lua_type == LUA_TNUMBER {
            if lua::lua_isinteger(L, idx) == 1 {
                Ok(pxs_Var::new_i64(lua::lua_tointegerx(L, idx, std::ptr::null_mut())))
            } else {
                Ok(pxs_Var::new_f64(lua::lua_tonumberx(L, idx, std::ptr::null_mut())))
            }
        } else if lua_type == LUA_TBOOLEAN {
            let lua_bool = lua::lua_toboolean(L, idx);
            Ok(pxs_Var::new_bool(lua_bool == 1))
        } else if lua_type == LUA_TSTRING {
            let lua_string = lua::lua_tolstring(L, idx, std::ptr::null_mut());
            let rust_string = borrow_string!(lua_string);
            Ok(pxs_Var::new_string(rust_string.to_string()))
        } else if lua_type == LUA_TFUNCTION {
            // Register the lua value.
            let reference = LuaReference::new();
            reference.push();
            Ok(pxs_Var::new_function(reference.into_void(), Some(free_lua_mem)))
        } else if lua_type == LUA_TTABLE {
            // Check length
            let t_length = lua::lua_rawlen(L, idx);

            if t_length == 0 {
                // Register table
                let reference = LuaReference::new();
                reference.push();
                Ok(pxs_Var::new_object(pxs_VarObject::new_lang_only(reference.into_void()), Some(free_lua_mem)))
            } else {
                // List dayo!
                let mut values = vec![];
                for i in 1..=t_length {
                    // Push it to stack
                    lua_geti(L, idx, i as i64);
                    let val = from_lua(-1)?;
                    lua_pop(L, 1);
                    values.push(val);
                }
                // Convert into list
                Ok(pxs_Var::new_list_with(values))
            }
        } else if lua_type == LUA_TNONE {
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
            pxs_VarType::pxs_Int64 => lua::lua_pushinteger(L, var.get_i64()?),
            pxs_VarType::pxs_UInt64 => lua::lua_pushinteger(L, var.get_u64()? as i64),
            pxs_VarType::pxs_String => {
                let mut cstring = CStringSafe::new();
                let contents = var.get_string()?;
                lua::lua_pushstring(L, cstring.new_string(&contents));
            },
            pxs_VarType::pxs_Bool => {
                let b = if var.get_bool()? {
                    1
                }else {
                    0
                };
                lua::lua_pushboolean(L, b);
            },
            pxs_VarType::pxs_Float64 => lua::lua_pushnumber(L, var.get_f64()?),
            pxs_VarType::pxs_Null => lua::lua_pushnil(L),
            pxs_VarType::pxs_Object => {
                let object = var.get_object_ptr();
                let reference= LuaReference::from_borrow_void(object);
                reference.push();
            },
            pxs_VarType::pxs_HostObject => {
                // Get the `PixelObject`
                let idx = var.value.host_object_val;
                let pixel_object = get_object(idx).unwrap();
                let lang_ptr_is_null = pixel_object.lang_ptr.lock().unwrap().is_null();
                if lang_ptr_is_null {
                    // Create the table for the first time
                    create_object(state, idx, Arc::clone(&pixel_object));

                    // Add table ptr
                    let table_ptr = LuaReference::new();
                    pixel_object.update_lang_ptr(table_ptr.into_void());
                    pixel_object.update_pxs_free_method(free_lua_mem);
                }
                // Get PTR again.
                let lang_ptr = pixel_object.lang_ptr.lock().unwrap();
                // Get as table.
                let table = LuaReference::from_borrow_void(*lang_ptr);
                // Return table
                table.push();
            },
            pxs_VarType::pxs_List => {
                let items = &var.get_list().unwrap().vars;
                lua_createtable(L, items.len() as i32, 0);
                let table = lua_gettop(L);

                for i in 0..items.len() {
                    let item = &items[i];
                    push_lua_stack(item)?;
                    lua_rawseti(L, table, (i + 1) as i64);
                }
                // for item in items.iter() {
                //     // Push to top of stack
                //     let idx = push_lua_stack(item)?;
                //     // Send it!
                //     lua_rawseti(L, table, idx as i64);
                // }
            },
            pxs_VarType::pxs_Function => {
                let val = var.get_function()?;
                let reference = LuaReference::from_borrow_void(val);
                reference.push();
            },
            pxs_VarType::pxs_Factory => {
                let factory = var.get_factory().unwrap();
                let res = factory.call(pxs_Runtime::pxs_Lua);
                push_lua_stack(&res)?;
            },
            pxs_VarType::pxs_Exception => {
                let msg = var.get_string()?;
                return pxs_error!("{msg}");
            },
            pxs_VarType::pxs_Map => {
                let map = var.get_map().unwrap();
                let keys = map.keys();
                lua_createtable(L, 0, keys.len() as i32);
                let table = lua_gettop(L);
                for k in keys {
                    push_lua_stack(k)?;
                    push_lua_stack(map.get_item(k).unwrap())?;
                    lua_settable(L, table);
                }
            },
        }

        Ok(lua_gettop(L))
    }
}
