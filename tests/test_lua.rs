#[cfg(test)]
mod tests {
    use std::{ffi::c_void, ptr, sync::Arc};

    use pixel_script::{
        lua::LuaScripting,
        shared::{PixelScript, PtrMagic, object::PixelObject, var::Var},
        *,
    };

    struct Person {
        name: String,
    }

    impl Person {
        pub fn new(n_name: String) -> Self {
            Person { name: n_name }
        }

        pub fn set_name(&mut self, n_name: String) {
            self.name = n_name;
        }

        pub fn get_name(&self) -> String {
            self.name.clone()
        }
    }

    impl PtrMagic for Person {}

    pub extern "C" fn free_person(ptr: *mut c_void) {
        let _ = unsafe { Person::from_borrow(ptr as *mut Person) };
    }

    pub extern "C" fn set_name(argc: usize, argv: *mut *mut Var, _opaque: *mut c_void) -> *mut Var {
        unsafe {
            let args = Var::slice_raw(argv, argc);
            // Get ptr
            let pixel_object_var = Var::from_borrow(args[0]);
            let host_ptr = pixel_object_var.get_host_ptr();
            let p = Person::from_borrow(host_ptr as *mut Person);

            let name = Var::from_borrow(args[1]);
            p.set_name(name.get_string().unwrap().clone());

            Var::into_raw(Var::new_null())
        }
    }

    pub extern "C" fn get_name(argc: usize, argv: *mut *mut Var, _opaque: *mut c_void) -> *mut Var {
        unsafe {
            let args = Var::slice_raw(argv, argc);
            // Get ptr
            let pixel_object_var = Var::from_borrow(args[0]);
            let host_ptr = pixel_object_var.get_host_ptr();
            let p = Person::from_borrow(host_ptr as *mut Person);

            Var::new_string(p.get_name().clone()).into_raw()
        }
    }

    pub extern "C" fn new_person(
        argc: usize,
        argv: *mut *mut Var,
        opaque: *mut c_void,
    ) -> *mut Var {
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);
            let p_name = Var::from_borrow(args[0]);
            let p_name = p_name.get_string().unwrap();
            let p = Person::new(p_name.clone());

            let ptr = Person::into_raw(p) as *mut c_void;
            let mut pixel_object = PixelObject::new(ptr, free_person);
            pixel_object.add_callback("set_name", set_name, opaque);
            pixel_object.add_callback("get_name", get_name, opaque);

            let pixel_arc = Arc::new(pixel_object);

            // Save...
            let idx = LuaScripting::save_object(Arc::clone(&pixel_arc));

            Var::into_raw(Var::new_host_object(idx))
        }
    }

    // Testing callbacks
    pub extern "C" fn print_wrapper(
        argc: usize,
        argv: *mut *mut Var,
        _opaque: *mut c_void,
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
        _opaque: *mut c_void,
    ) -> *mut Var {
        // Assumes n1 and n2
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);

            let n1 = Var::from_borrow(args[0]);
            let n2 = Var::from_borrow(args[1]);

            Var::new_i64(n1.value.i64_val + n2.value.i64_val).into_raw()
        }
    }

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
    fn test_add_object() {
        LuaScripting::add_object("Person", new_person, ptr::null_mut());
    }

    #[test]
    fn test_execute() {
        test_add_variable();
        test_add_callback();
        test_add_module();
        test_add_object();
        let lua_code = r#"
            local msg = "Welcome, " .. name
            println(msg)

            local math = require("cmath")

            local result = math.add(math.n1, math.n2)
            println("Module result: " .. tostring(result))

            if result ~= 3 then
                error("Math, Expected 3, got " .. tostring(result))
            end

            local person = Person("Jordan")
            println(person.get_name())
            person.set_name("Jordan Castro")
            println(person:get_name())
        "#;
        let err = LuaScripting::execute(lua_code, "<test>");

        assert!(err.is_empty(), "Error is not empty: {}", err);
    }
}
