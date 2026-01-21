// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_repl -- --nocapture --test-threads=1
#[cfg(test)]
mod tests {
    use std::{
        ffi::{CStr, CString, c_char, c_void},
        ptr,
    };

    use pixelscript::{
        lua::LuaScripting,
        python::PythonScripting,
        shared::{PixelScript, PixelScriptRuntime, PtrMagic, pxs_DirHandle, var::pxs_Var},
        *,
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

    pub extern "C" fn set_name(argc: usize, argv: *mut *mut pxs_Var, _opaque: *mut c_void) -> *mut pxs_Var {
        unsafe {
            let args = pxs_Var::slice_raw(argv, argc);
            // Get ptr
            let pixel_object_var = pxs_Var::from_borrow(args[1]);
            let host_ptr = pixel_object_var.get_host_ptr();
            let p = Person::from_borrow(host_ptr as *mut Person);

            // Check if first arg is self or nme
            let name = {
                let first_arg = pxs_Var::from_borrow(args[2]);
                if first_arg.is_string() {
                    first_arg
                } else {
                    pxs_Var::from_borrow(args[3])
                }
            };

            p.set_name(name.get_string().unwrap().clone());

            pxs_Var::into_raw(pxs_Var::new_null())
        }
    }

    pub extern "C" fn get_name(argc: usize, argv: *mut *mut pxs_Var, _opaque: *mut c_void) -> *mut pxs_Var {
        unsafe {
            let args = pxs_Var::slice_raw(argv, argc);

            // Get ptr
            let pixel_object_var = pxs_Var::from_borrow(args[1]);
            let host_ptr = pixel_object_var.get_host_ptr();
            let p = Person::from_borrow(host_ptr as *mut Person);

            pxs_Var::new_string(p.get_name().clone()).into_raw()
        }
    }

    pub extern "C" fn new_person(
        argc: usize,
        argv: *mut *mut pxs_Var,
        opaque: *mut c_void,
    ) -> *mut pxs_Var {
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);
            let p_name = pxs_Var::from_borrow(args[1]);
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
        argv: *mut *mut pxs_Var,
        _opaque: *mut c_void,
    ) -> *mut pxs_Var {
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);

            let runtime = args[0];

            let mut string = String::new();
            for i in 1..argc {
                let var = pixelscript_var_tostring(runtime, args[i]);
                if let Ok(s) = (*var).get_string() {
                    string.push_str(format!("{s} ").as_str());
                }
                pixelscript_free_var(var);
            }

            println!("From Runtime: {string}");
        }

        pxs_Var::new_null().into_raw()
    }

    pub extern "C" fn add_wrapper(
        argc: usize,
        argv: *mut *mut pxs_Var,
        _opaque: *mut c_void,
    ) -> *mut pxs_Var {
        // Assumes n1 and n2
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);

            let n1 = pxs_Var::from_borrow(args[1]);
            let n2 = pxs_Var::from_borrow(args[2]);

            pxs_Var::new_i64(n1.value.i64_val + n2.value.i64_val).into_raw()
        }
    }

    pub extern "C" fn sub_wrapper(
        argc: usize,
        argv: *mut *mut pxs_Var,
        _opaque: *mut c_void,
    ) -> *mut pxs_Var {
        // Assumes n1 and n2
        unsafe {
            let args = std::slice::from_raw_parts(argv, argc);

            let n1 = pxs_Var::from_borrow(args[1]);
            let n2 = pxs_Var::from_borrow(args[2]);

            pxs_Var::new_i64(n1.value.i64_val - n2.value.i64_val).into_raw()
        }
    }

    unsafe extern "C" fn file_loader(file_path: *const c_char) -> *mut c_char {
        let file_path = unsafe { CStr::from_ptr(file_path).to_str().unwrap() };

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

    unsafe extern "C" fn dir_reader(dir_path: *const c_char) -> pxs_DirHandle {
        let dir_path = unsafe { CStr::from_ptr(dir_path).to_str().unwrap() };

        if dir_path.is_empty() {
            return pxs_DirHandle::empty();
        }

        // Check if dir exists
        let dir_exists = std::fs::exists(dir_path).unwrap();
        if !dir_exists {
            return pxs_DirHandle::empty();
        }

        // Load dir
        let files = std::fs::read_dir(dir_path).unwrap();
        let mut result = vec![];

        for f in files {
            let entry = f.unwrap();
            result.push(entry.file_name().into_string().unwrap());
        }

        // 1. Convert Strings to CStrings, then to raw pointers
        // We use .into_raw() so Rust surrenders ownership and doesn't free the memory
        let mut c_ptrs: Vec<*mut c_char> = result
            .into_iter()
            .map(|s| CString::new(s).unwrap().into_raw())
            .collect();

        // 2. Get a pointer to the array of pointers
        // We get the pointer to the underlying buffer of the Vec
        let argv: *mut *mut c_char = c_ptrs.as_mut_ptr();
        let argc = c_ptrs.len();
        pxs_DirHandle {
            length: argc,
            values: argv,
        }
    }

    // // #[test]
    // fn test_add_variable() {
    //     println!("Inside test add variable");
    //     pixelscript_initialize();
    //     let name = create_raw_string!("name");
    //     let jordan = create_raw_string!("Jordan");
    //     let var = pixelscript_var_newstring(jordan);
    //     println!("Before add variable");
    //     pixelscript_add_variable(name, var);
    //     println!("After add variable");
    //     free_raw_string!(name);
    //     println!("Freed strings");
    // }

    // // #[test]
    // fn test_add_callback() {
    //     println!("Inside Test add callback");
    //     pixelscript_initialize();
    //     let name = create_raw_string!("println");
    //     pixelscript_add_callback(name, print_wrapper, ptr::null_mut());
    //     free_raw_string!(name);
    // }

    // #[test]
    fn test_add_module() {
        println!("Inside Test add module");
        pixelscript_initialize();
        let module_name = create_raw_string!("pxs");
        let module = pixelscript_new_module(module_name);
        // Save methods
        let add_name = create_raw_string!("add");
        let n1_name = create_raw_string!("n1");
        let n2_name: *mut i8 = create_raw_string!("n2");
        pixelscript_add_callback(module, add_name, add_wrapper, ptr::null_mut());
        let n1 = pixelscript_var_newint(1);
        let n2 = pixelscript_var_newint(2);
        pixelscript_add_variable(module, n1_name, n1);
        pixelscript_add_variable(module, n2_name, n2);

        let name = create_raw_string!("print");
        pixelscript_add_callback(module, name, print_wrapper, ptr::null_mut());
        let var_name = create_raw_string!("name");
        let jordan = create_raw_string!("Jordan C");
        let var = pixelscript_var_newstring(jordan);
        pixelscript_add_variable(module, var_name, var);

        let object_name = create_raw_string!("Person");
        pixelscript_add_object(module, object_name, new_person, ptr::null_mut());

        // Add a inner module
        let math_module_name = create_raw_string!("math");
        let math_module = pixelscript_new_module(math_module_name);

        // Add a sub function
        let sub_name = create_raw_string!("sub");
        pixelscript_add_callback(math_module, sub_name, sub_wrapper, ptr::null_mut());

        pixelscript_add_submodule(module, math_module);
        pixelscript_add_module(module);

        free_raw_string!(module_name);
        free_raw_string!(add_name);
        free_raw_string!(n1_name);
        free_raw_string!(n2_name);
        free_raw_string!(object_name);
        free_raw_string!(name);
        free_raw_string!(var_name);
        free_raw_string!(math_module_name);
        free_raw_string!(sub_name);
    }

    // // #[test]
    // fn test_add_object() {
    //     pixelscript_initialize();
    //     let object_name = create_raw_string!("Person");
    //     pixelscript_add_object(object_name, new_person, ptr::null_mut());
    //     free_raw_string!(object_name);
    // }

    #[test]
    fn test_execute() {
        println!("Test starting");
        pixelscript_initialize();

        // test_add_variable();
        // println!("Var");
        // test_add_callback();
        // println!("Callback");
        test_add_module();
        // println!("Module");
        // test_add_object();
        // println!("Object");

        pixelscript_set_file_reader(file_loader);
        pixelscript_set_dir_reader(dir_reader);

        let runtime = PixelScriptRuntime::Lua;
        let mut lines = vec![];
        loop {
            let mut input = String::new(); // Create an empty, mutable String
            std::io::stdin()
                .read_line(&mut input) // Read the line and store it in 'input'
                .expect("Failed to read line"); // Handle potential errors

            if input.contains("quit") {
                break;
            } else if input.contains("run") {
                let full_lines = lines.join("\n");
                let err = match runtime {
                    PixelScriptRuntime::Lua => LuaScripting::execute(&full_lines, "<test_repl>"),
                    PixelScriptRuntime::Python => {
                        PythonScripting::execute(&full_lines, "<test_repl>")
                    }
                    PixelScriptRuntime::JavaScript => todo!(),
                    PixelScriptRuntime::Easyjs => todo!(),
                };

                if !err.is_empty() {
                    println!("Repl error is: {err}");
                }
            } else {
                lines.push(input.clone());
            }
        }

        pixelscript_finalize();
    }
}
