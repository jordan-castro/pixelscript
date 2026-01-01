#[cfg(test)]
mod tests {
    use std::{ffi::c_void, ptr};

    use pixel_mods::{shared::var::Var, *};

    // Testing callbacks
    pub extern "C" fn print_wrapper(
        argc: usize,
        argv: *mut *mut Var,
        opaque: *mut c_void,
    ) -> *mut Var {
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);

            let var_ptr = args[0];

            let var_ref = &*var_ptr;

            if let Ok(msg) = var_ref.get_string() {
                println!("Lua sent: {}", msg);
            }
        }

        Box::into_raw(Box::new(Var::new_null()))
    }

    #[test]
    fn test_add_variable() {
        lua::var::add_variable("name", Var::new_string(String::from("Jordan")));
    }

    #[test]
    fn test_add_callback() {
        lua::func::add_callback("println", print_wrapper, ptr::null_mut());
    }

    #[test]
    fn test_execute() {
        test_add_variable();
        test_add_callback();
        let lua_code = r#"
            local msg = "Welcome, " .. name
            println(msg)
            println("FFI Callback Success!")
        "#;
        let err = lua::execute(lua_code, "<test>");

        assert!(err.is_empty(), "Error is not empty: {}", err);
    }
}
