use mlua::{IntoLua, Value::Nil};

// Pure Rust goes here
use crate::{
    lua::get_state,
    shared::var::{Var, VarType},
};

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
        }
    }
}

/// Add a variable by name to __main__ in lua.
pub fn add_variable(name: &str, variable: Var) {
    let state = get_state();

    match variable.tag {
        crate::shared::var::VarType::Int32 => {
            state
                .engine
                .globals()
                .set(
                    name,
                    variable.get_i32().expect("Could not unwrap i32 from Var."),
                )
                .expect("Could not set variable.");
        }
        crate::shared::var::VarType::Int64 => {
            state
                .engine
                .globals()
                .set(
                    name,
                    variable.get_i64().expect("Could not unwrap i64 from Var."),
                )
                .expect("Could not set variable.");
        }
        crate::shared::var::VarType::UInt32 => {
            state
                .engine
                .globals()
                .set(
                    name,
                    variable.get_u32().expect("Could not unwrap u32 from Var."),
                )
                .expect("Could not set variable.");
        }
        crate::shared::var::VarType::UInt64 => {
            state
                .engine
                .globals()
                .set(
                    name,
                    variable.get_u64().expect("Could not unwrap u64 from Var."),
                )
                .expect("Could not set variable.");
        }
        crate::shared::var::VarType::String => {
            state
                .engine
                .globals()
                .set(
                    name,
                    variable
                        .get_string()
                        .expect("Could not unwrap String from Var"),
                )
                .expect("Could not set variable.");
        }
        crate::shared::var::VarType::Bool => {
            state
                .engine
                .globals()
                .set(
                    name,
                    variable
                        .get_bool()
                        .expect("Could not unwrap bool from Var."),
                )
                .expect("Could not set varible.");
        }
        crate::shared::var::VarType::Float32 => {
            state
                .engine
                .globals()
                .set(
                    name,
                    variable.get_f32().expect("Could not unwrap f32 from Var."),
                )
                .expect("Could not set variable.");
        }
        crate::shared::var::VarType::Float64 => {
            state
                .engine
                .globals()
                .set(
                    name,
                    variable.get_f64().expect("Could not unwrap f64 from Var."),
                )
                .expect("Could not set variable.");
        }
        crate::shared::var::VarType::Null => {
            state.engine.globals().set(name, Nil).expect("Could not set variable.");
        }
    }

    // Listo!
}
