use std::ffi::c_void;

use mlua::prelude::*;
// use mlua::{Integer, IntoLua, Lua, MultiValue, Value::Nil, Variadic};

use crate::{lua::get_state, shared::{func::{Func, get_function_lookup}, var::Var}};

/// For internal use since modules also need to use the same logic for adding a Lua callback.
pub(super) fn internal_add_callback(lua: &Lua, func: Func, opaque: *mut c_void, obj: Option<i32>) -> LuaFunction {
    // Save the function
    let mut function_lookup = get_function_lookup();
    let idx = function_lookup.add_function(func, opaque);

    lua.create_function(move |lua, args: LuaMultiValue| {
        // Convert args -> argv for pixelmods
        let mut argv: Vec<Var> = vec![];

        let (func, opaque) = {
            let function_lookup = get_function_lookup();
            let data = function_lookup.get_function(idx).unwrap();

            (data.func, data.opaque)
        };

        // If a obj is passed
        if let Some(obj) = obj {
            // Add the pointer.
            argv.push(Var::new_i64(obj as i64));
        }

        for arg in args {
            argv.push(Var::from_lua(arg, lua).expect("Could not convert value into Var from Lua."));
            // match arg {
            //     mlua::Value::Boolean(b) => {
            //         argv.push(Var::new_bool(b));
            //     },
            //     mlua::Value::Integer(i) => {
            //         // TODO: accept u32, u64, or i32
            //         argv.push(Var::new_i64(i));
            //     },
            //     mlua::Value::Number(f) => {
            //         // TODO: accept f32 too
            //         argv.push(Var::new_f64(f));
            //     },
            //     mlua::Value::String(s) => {
            //         argv.push(Var::new_string(s.to_string_lossy()));  
            //     },
            //     _ => {
            //         // For now default all other values to null.
            //         // TODO: Wrap values correctly.
            //         argv.push(Var::new_null());
            //     }
            //     // mlua::Value::LightUserData(light_user_data) => todo!(),
            //     // mlua::Value::Nil => todo!(),
            //     // mlua::Value::Table(table) => todo!(),
            //     // mlua::Value::Function(function) => todo!(),
            //     // mlua::Value::Thread(thread) => todo!(),
            //     // mlua::Value::UserData(any_user_data) => todo!(),
            //     // mlua::Value::Error(error) => todo!(),
            //     // mlua::Value::Other(value_ref) => todo!(),
            // }
        }        

        let argc = argv.len();
        // Convert argv into a pointer array
        let ptrs = Var::make_pointer_array(argv);

        unsafe {
            // Pass into the function
            let res_ptr = func(argc, ptrs, opaque);

            // Now free ptrs (since they are owned by Lua now)
            Var::free_pointer_array(ptrs, argc);

            // Convert *mut Var into Var
            if res_ptr.is_null() {
                Ok(LuaNil)
            } else {
                // Recontstruct the Var
                let res_box = Box::from_raw(res_ptr);

                // Convert and pass memory into LUA
                let lua_val = res_box.into_lua(lua);

                lua_val

                // Memory will drop here, and Var will be automatically freed!
            }
        }
    }).expect("Could not create lua function")
}

/// Add a callback to lua __main__ context.
pub(super) fn add_callback(name: &str, func: Func, opaque: *mut c_void) {
    let state = get_state();
    let lua_func = internal_add_callback(&state.engine, func, opaque, None);
    state.engine.globals().set(name, lua_func).expect("Could not add callback to Lua.");
}