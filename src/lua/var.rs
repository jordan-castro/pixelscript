use std::{ffi::c_void, sync::Arc};

// use mlua::{IntoLua, Lua};
use mlua::prelude::*;

// Pure Rust goes here
use crate::{
    lua::create_object,
    shared::{
        object::get_object_lookup,
        var::{Var, VarType},
    },
};

// TODO: use LuaUnsigned for u32 and u64 maybe.
impl IntoLua for Var {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        // Convert the Rust/C type into Lua. Once it's in LUA we can free our memory, lua copies it and handles it from here on out.
        match self.tag {
            VarType::Int32 => Ok(mlua::Value::Integer(self.get_i32().unwrap() as i64)),
            VarType::Int64 => Ok(mlua::Value::Integer(self.get_i64().unwrap())),
            VarType::UInt32 => Ok(mlua::Value::Integer(self.get_u32().unwrap() as i64)),
            VarType::UInt64 => Ok(mlua::Value::Integer(self.get_u64().unwrap() as i64)),
            VarType::String => {
                let contents = self.get_string().unwrap();
                let lua_str = lua.create_string(contents).expect("test");

                Ok(mlua::Value::String(lua_str))
            }
            VarType::Bool => Ok(mlua::Value::Boolean(self.get_bool().unwrap())),
            VarType::Float32 => Ok(mlua::Value::Number(self.get_f32().unwrap() as f64)),
            VarType::Float64 => Ok(mlua::Value::Number(self.get_f64().unwrap())),
            VarType::Null => Ok(mlua::Value::Nil),
            VarType::Object => {
                unsafe {
                    // This MUST BE A TABLE!
                    let table_ptr = self.value.object_val as *const LuaTable;
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
                    let idx = self.value.host_object_val;
                    let object_lookup = get_object_lookup();
                    let pixel_object = object_lookup.get_object(idx).unwrap().clone();
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
}

impl FromLua for Var {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
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
}

/// Add a variable by name to __main__ in lua.
pub fn add_variable(context: &Lua, name: &str, variable: Var) {
    context
        .globals()
        .set(
            name,
            variable
                .into_lua(context)
                .expect("Could not unwrap LUA vl from Var."),
        )
        .expect("Could not add variable to Lua global context.");
    // Listo!
}
