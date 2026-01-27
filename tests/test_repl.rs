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
        shared::{
            PixelScript, PtrMagic, pxs_DirHandle, pxs_Runtime,
            var::{pxs_Var, pxs_VarT},
        },
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

    struct TodoList {
        items: Vec<String>,
    }

    impl TodoList {
        pub fn new(initial_items: Vec<String>) -> Self {
            TodoList {
                items: initial_items,
            }
        }

        pub fn add_item(&mut self, item: &str) {
            self.items.push(item.to_string().clone());
        }

        pub fn get_item(&mut self, index: usize) -> Option<String> {
            self.items.get(index).cloned()
        }
    }

    impl PtrMagic for TodoList {}

    pub extern "C" fn free_todolist(ptr: *mut c_void) {
        println!("Freeing TODOList!");
        let _ = unsafe { TodoList::from_borrow(ptr as *mut TodoList) };
    }

    pub extern "C" fn get_item(args: *mut pxs_Var, _opaque: pxs_Opaque) -> pxs_VarT {
        unsafe {
            let pxsobject = borrow_var!(pxs_listget(args, 1));

            // Let index
            let index = borrow_var!(pxs_listget(args, 2));

            // Get TodoList
            let todolist =
                unsafe { TodoList::from_borrow(pxs_gethost(pxsobject) as *mut TodoList) };

            // Get at index
            let item = todolist.get_item(pxs_getint(index) as usize);

            if let Some(item) = item {
                let raw_string = create_raw_string!(item);
                let result = pxs_newstring(raw_string);
                free_raw_string!(raw_string);
                result
            } else {
                let raw_string = create_raw_string!("");
                let result = pxs_newstring(raw_string);
                free_raw_string!(raw_string);
                result
            }
        }
    }

    pub extern "C" fn add_item(args: pxs_VarT, _opaque: pxs_Opaque) -> pxs_VarT {
        unsafe {
            let pxsobject = borrow_var!(pxs_listget(args, 1));
            // item
            let item = borrow_var!(pxs_listget(args, 2));

            // Derefernce
            let todolist =
                unsafe { TodoList::from_borrow(pxs_gethost(pxsobject) as *mut TodoList) };

            // Get string
            let item_str = pxs_getstring(item);
            let string = borrow_string!(item_str).to_string().clone();
            pxs_freestr(item_str);

            // Now add item
            todolist.add_item(&string);

            pxs_newnull()
        }
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

    pub extern "C" fn set_name(args: *mut pxs_Var, _opaque: *mut c_void) -> *mut pxs_Var {
        unsafe {
            // Get ptr
            let pixel_object_var = pxs_Var::from_borrow(pxs_listget(args, 1));
            let host_ptr = pixel_object_var.get_host_ptr();
            let p = Person::from_borrow(host_ptr as *mut Person);

            // Check if first arg is self or nme
            let name = {
                let first_arg = pxs_Var::from_borrow(pxs_listget(args, 2));
                if first_arg.is_string() {
                    first_arg
                } else {
                    pxs_Var::from_borrow(pxs_listget(args, 3))
                }
            };

            p.set_name(name.get_string().unwrap().clone());

            pxs_Var::into_raw(pxs_Var::new_null())
        }
    }

    pub extern "C" fn get_name(args: *mut pxs_Var, _opaque: *mut c_void) -> *mut pxs_Var {
        unsafe {
            // Get ptr
            let pixel_object_var = pxs_Var::from_borrow(pxs_listget(args, 1));
            let host_ptr = pixel_object_var.get_host_ptr();
            let p = Person::from_borrow(host_ptr as *mut Person);

            pxs_Var::new_string(p.get_name().clone()).into_raw()
        }
    }

    pub extern "C" fn new_person(args: *mut pxs_Var, opaque: *mut c_void) -> *mut pxs_Var {
        unsafe {
            let p_name = pxs_Var::from_borrow(pxs_listget(args, 1));
            let p_name = p_name.get_string().unwrap();
            let p = Person::new(p_name.clone());
            let typename = create_raw_string!("Person");

            let ptr = Person::into_raw(p) as *mut c_void;
            let pixel_object = pxs_newobject(ptr, free_person, typename);
            let set_name_raw = create_raw_string!("set_name");
            let get_name_raw = create_raw_string!("get_name");
            pxs_object_addfunc(pixel_object, set_name_raw, set_name, opaque);
            pxs_object_addfunc(pixel_object, get_name_raw, get_name, opaque);
            // Save...
            let var = pxs_newhost(pixel_object);

            free_raw_string!(set_name_raw);
            free_raw_string!(get_name_raw);
            free_raw_string!(typename);
            var
        }
    }

    // Testing callbacks
    pub extern "C" fn print_wrapper(args: *mut pxs_Var, _opaque: *mut c_void) -> *mut pxs_Var {
        unsafe {
            let runtime = pxs_listget(args, 0);

            let mut string = String::new();
            for i in 1..pxs_listlen(args) {
                let var = pxs_tostring(runtime, pxs_listget(args, i));
                if let Ok(s) = (*var).get_string() {
                    string.push_str(format!("{s} ").as_str());
                }
                pxs_freevar(var);
            }

            println!("From Runtime: {string}");
        }

        pxs_Var::new_null().into_raw()
    }

    pub extern "C" fn add_wrapper(args: *mut pxs_Var, _opaque: *mut c_void) -> *mut pxs_Var {
        // Assumes n1 and n2
        unsafe {
            let n1 = pxs_Var::from_borrow(pxs_listget(args, 1));
            let n2 = pxs_Var::from_borrow(pxs_listget(args, 2));

            pxs_Var::new_i64(n1.value.i64_val + n2.value.i64_val).into_raw()
        }
    }

    pub extern "C" fn sub_wrapper(args: *mut pxs_Var, _opaque: *mut c_void) -> *mut pxs_Var {
        // Assumes n1 and n2
        unsafe {
            let n1 = pxs_Var::from_borrow(pxs_listget(args, 1));
            let n2 = pxs_Var::from_borrow(pxs_listget(args, 2));

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

    fn test_add_module() {
        println!("Inside Test add module");
        pxs_initialize();
        let module_name = create_raw_string!("pxs");
        let module = pxs_newmod(module_name);
        // Save methods
        let add_name = create_raw_string!("add");
        let n1_name = create_raw_string!("n1");
        let n2_name: *mut i8 = create_raw_string!("n2");
        pxs_addfunc(module, add_name, add_wrapper, ptr::null_mut());
        let n1 = pxs_newint(1);
        let n2 = pxs_newint(2);
        pxs_addvar(module, n1_name, n1);
        pxs_addvar(module, n2_name, n2);

        let name = create_raw_string!("print");
        pxs_addfunc(module, name, print_wrapper, ptr::null_mut());
        let var_name = create_raw_string!("name");
        let jordan = create_raw_string!("Jordan C");
        let var = pxs_newstring(jordan);
        pxs_addvar(module, var_name, var);

        let object_name = create_raw_string!("Person");
        pxs_addobject(module, object_name, new_person, ptr::null_mut());

        // Add a inner module
        let math_module_name = create_raw_string!("math");
        let math_module = pxs_newmod(math_module_name);

        // // Add the todolist to the math module.
        // let todolist = TodoList::new(vec![]).into_raw();
        // let typename = create_raw_string!("TodoList");
        // let object = pxs_newobject(todolist as *mut c_void, free_todolist, typename);
        // free_raw_string!(typename);
        // // Add methods
        // let func_name = create_raw_string!("add_item");
        // pxs_object_addfunc(object, func_name, add_item, ptr::null_mut());
        // free_raw_string!(func_name);
        // let func_name = create_raw_string!("get_item");
        // free_raw_string!(func_name);
        // pxs_object_addfunc(object, func_name, get_item, ptr::null_mut());

        // println!("Before adding new variable");
        // // Set variable
        // let var_name2 = create_raw_string!("todo");
        // let host_object = pxs_newhost(object);
        // if host_object.is_null() {
        //     println!("Host is null");
        // } else {
        //     println!("Host is not null my man");
        // }
        // pxs_addvar(math_module, var_name2, host_object);
        // free_raw_string!(var_name2);

        // println!("After addiing");
        // Add a sub function
        let sub_name = create_raw_string!("sub");
        pxs_addfunc(math_module, sub_name, sub_wrapper, ptr::null_mut());

        pxs_add_submod(module, math_module);
        pxs_addmod(module);

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

    #[test]
    fn test_execute() {
        println!("Test starting");
        pxs_initialize();
        test_add_module();

        pxs_set_filereader(file_loader);
        pxs_set_dirreader(dir_reader);

        let runtime = pxs_Runtime::pxs_Python;
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
                    pxs_Runtime::pxs_Lua => LuaScripting::execute(&full_lines, "<test_repl>"),
                    pxs_Runtime::pxs_Python => PythonScripting::execute(&full_lines, "<test_repl>"),
                    pxs_Runtime::pxs_JavaScript => todo!(),
                    pxs_Runtime::pxs_Easyjs => todo!(),
                    pxs_Runtime::pxs_RustPython => todo!(),
                    _ => todo!(), // pxs_Runtime::LuaJit => todo!(),
                };

                if !err.is_empty() {
                    println!("Repl error is: {err}");
                }
            } else {
                lines.push(input.clone());
            }
        }

        pxs_finalize();
    }
}
