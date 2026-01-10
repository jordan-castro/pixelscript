#[cfg(test)]
mod tests {
    use std::{
        ffi::{CStr, CString, c_char, c_void},
        ptr,
    };

    use pixelscript::{python::PythonScripting, shared::{PixelScript, PtrMagic, var::Var}, *
    };

    /// Create a raw string from &str.
    ///
    /// Remember to FREE THIS!
    macro_rules! create_raw_string {
        ($rstr:expr) => {{ CString::new($rstr).unwrap().into_raw() }};
    }

    /// Free a raw sring
    macro_rules! free_raw_string {
        ($rptr:expr) => {{
            if !$rptr.is_null() {
                unsafe {
                    let _ = std::ffi::CString::from_raw($rptr);
                }
            }
        }};
    }

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
            let pixel_object_var = Var::from_borrow(args[1]);
            let host_ptr = pixel_object_var.get_host_ptr();
            let p = Person::from_borrow(host_ptr as *mut Person);

            // Check if first arg is self or nme
            let name = {
                let first_arg = Var::from_borrow(args[2]);
                if first_arg.is_string() {
                    first_arg
                } else {
                    Var::from_borrow(args[3])
                }
            };

            p.set_name(name.get_string().unwrap().clone());

            Var::into_raw(Var::new_null())
        }
    }

    pub extern "C" fn get_name(argc: usize, argv: *mut *mut Var, _opaque: *mut c_void) -> *mut Var {
        unsafe {
            let args = Var::slice_raw(argv, argc);

            // Get ptr
            let pixel_object_var = Var::from_borrow(args[1]);
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
            let p_name = Var::from_borrow(args[1]);
            let p_name = p_name.get_string().unwrap();
            let p = Person::new(p_name.clone());
            let typename = create_raw_string!("Person");

            let ptr = Person::into_raw(p) as *mut c_void;
            let pixel_object = pixelscript_new_object(ptr, free_person, typename);
            let set_name_raw = create_raw_string!("set_name");
            let get_name_raw = create_raw_string!("get_name");
            pixelscript_object_add_callback(pixel_object, set_name_raw, set_name, opaque);
            pixelscript_object_add_callback(pixel_object, get_name_raw, get_name, opaque);
            // Save...
            let var = pixelscript_var_newhost_object(pixel_object);

            free_raw_string!(set_name_raw);
            free_raw_string!(get_name_raw);
            free_raw_string!(typename);
            var
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

            let var_ptr = Var::from_borrow(args[1]);

            if let Ok(msg) = var_ptr.get_string() {
                println!("Python sent: {}", msg);
            } else {
                println!("Could not get a string?");
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

            let n1 = Var::from_borrow(args[1]);
            let n2 = Var::from_borrow(args[2]);

            Var::new_i64(n1.value.i64_val + n2.value.i64_val).into_raw()
        }
    }

    unsafe extern "C" fn file_loader(file_path: *const c_char) -> *mut c_char {
        let file_path = unsafe{ CStr::from_ptr(file_path).to_str().unwrap() };

        if file_path.is_empty() {
            return create_raw_string!("");
        }

        let file_exists = std::fs::exists(file_path).unwrap();

        if !file_exists {
            return create_raw_string!("");
        }

        // Read file
        let contents = std::fs::read_to_string(file_path).unwrap();

        // Return contents
        create_raw_string!(contents)
    }

    #[test]
    fn test_add_variable() {
        pixelscript_initialize();
        PythonScripting::add_variable("name", &Var::new_string(String::from("Jordan")));
    }
    #[test]
   fn test_add_callback() {
        pixelscript_initialize();
        let name = create_raw_string!("println");
        pixelscript_add_callback(name, print_wrapper, ptr::null_mut());
        free_raw_string!(name);
    }

    #[test]
    fn test_add_module() {
        pixelscript_initialize();
        let module_name = create_raw_string!("ps_math");
        let module = pixelscript_new_module(module_name);
        // Save methods
        let add_name = create_raw_string!("add");
        let n1_name = create_raw_string!("n1");
        let n2_name = create_raw_string!("n2");
        pixelscript_module_add_callback(module, add_name, add_wrapper, ptr::null_mut());
        pixelscript_module_add_variable(module, n1_name, &Var::new_i64(1));
        pixelscript_module_add_variable(module, n2_name, &Var::new_i64(2));

        pixelscript_add_module(module);

        free_raw_string!(module_name);
        free_raw_string!(add_name);
        free_raw_string!(n1_name);
        free_raw_string!(n2_name);
    }

    #[test]
    fn test_add_object() {
        pixelscript_initialize();
        let object_name = create_raw_string!("Person");
        pixelscript_add_object(object_name, new_person, ptr::null_mut());
        free_raw_string!(object_name);
    }

    #[test]
    fn test_execute() {
        pixelscript_initialize();

        test_add_variable();
        test_add_callback();
        test_add_module();
        test_add_object();

        pixelscript_set_file_reader(file_loader);

        let py_code = r#"
import ps_math
from pad.ft_object import function_from_outside 

function_from_outside() # Should print something

msg = "Welcome " + name
println(msg)

result = ps_math.add(ps_math.n1, ps_math.n2)
println(f"Module result: {result}")

if result != 3:
    raise "Math, Expected 3, got " + str(result)

person = Person("Jordan")

println(person.get_name())
person.set_name("Jordan Castro")
println(person.get_name())

println(type(person).__name__)
println(type(Person).__name__)
        "#;
        let err = PythonScripting::execute(py_code, "<test>");
        assert!(err.is_empty(), "Python Error is not empty: {}", err);

        pixelscript_finalize();
    }
}
