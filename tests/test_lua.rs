#[cfg(test)]
mod tests {
    use std::{ffi::c_void, ptr, sync::Arc};

    use pixel_script::{lua::LuaScripting, shared::{PixelScript, PtrMagic, var::Var}, *};

    struct Person {
        name: String
    }

    impl Person {
        pub fn new(n_name:String) -> Self {
            Person {
                name: n_name
            }
        }

        pub fn set_name(&mut self, n_name:String) {
            self.name = n_name;
        }

        pub fn get_name(&self) -> String {
            self.name.clone()
        }
    }

    pub extern "C" fn free_person(ptr: *mut c_void) {}

    pub extern "C" fn new_person(argc: usize, argv: *mut *mut Var, opaque: *mut c_void) -> *mut Var {
        unsafe {
            // let p = Person::new();
            pixelscript_new_object(ptr, free_person)
        }
    }

    // Testing callbacks
    pub extern "C" fn print_wrapper(
        argc: usize,
        argv: *mut *mut Var,
        opaque: *mut c_void,
    ) -> *mut Var {
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);

            let var_ptr = Var::from_borrow(args[0]);

            if let Ok(msg) = var_ptr.get_string() {
                println!("Lua sent: {}", msg);
            }
        }

        Var::new_null().into_raw()
    }

    pub extern "C" fn add_wrapper(
        argc: usize,
        argv: *mut *mut Var,
        opaque: *mut c_void
    ) -> *mut Var {
        // Assumes n1 and n2
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);

            let n1 = Var::from_borrow(args[0]);
            let n2 = Var::from_borrow(args[1]);

            Var::new_i64(n1.value.i64_val + n2.value.i64_val).into_raw()
        }
    }

    // pub extern "C" fn greet_wrapper(
    //     argc: usize,
    //     argv: *mut *mut Var,
    //     opaque: *mut c_void
    // ) -> *mut Var {
    //     unsafe {
    //     obj = argc[0].obj
    //     obj.call('test')
    //     return Var::new_object()
    //     }
    // }

    #[test]
    fn test_add_variable() {
        LuaScripting::add_variable("name", &Var::new_string(String::from("Jordan")));
        // lua::var::add_variable("name", Var::new_string(String::from("Jordan")));
    }

    #[test]
    fn test_add_callback() {
        LuaScripting::add_callback("println", print_wrapper, ptr::null_mut());
    }

    #[test]
    fn test_add_module() {
        let mut module = shared::module::Module::new("cmath".to_string());
        module.add_callback("add", add_wrapper, ptr::null_mut());
        module.add_variable("n1", &Var::new_i64(1));
        module.add_variable("n2", &Var::new_i64(2));

        LuaScripting::add_module(Arc::new(module));
    }

    #[test]
    fn test_add_class() {
        let mut class = shared::class::PixelClass::new("Person".to_string());
        class.add_variable("name", &Var::new_string("Jordan".to_owned()));
        class.add_variable("age", &Var::new_i64(23));
        // class.add_callback("greet", func, opaque);
    }

    #[test]
    fn test_execute() {
        test_add_variable();
        test_add_callback();
        test_add_module();
        let lua_code = r#"
            local msg = "Welcome, " .. name
            println(msg)

            local math = require("cmath")

            local result = math.add(math.n1, math.n2)
            println("Module result: " .. tostring(result))

            if result ~= 3 then
                error("Math, Expected 3, got " .. tostring(result))
            end
            println("Yessir!")
        "#;
        let err = LuaScripting::execute(lua_code, "<test>");

        assert!(err.is_empty(), "Error is not empty: {}", err);
    }
}
