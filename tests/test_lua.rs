// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_lua --no-default-features --features "lua" -- --nocapture

#[cfg(test)]
mod tests {
    use std::{
        ffi::{CStr, CString, c_char, c_void},
        ptr,
        sync::Arc,
    };

    use pixelscript::{
        lua::LuaScripting,
        shared::{PixelScript, PtrMagic, object::pxs_PixelObject, pxs_Runtime, var::{pxs_Var, pxs_VarT}},
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

    pub extern "C" fn set_name(
        args: *mut pxs_Var,
        _opaque: *mut c_void,
    ) -> *mut pxs_Var {
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

    pub extern "C" fn get_name(
        args: *mut pxs_Var,
        _opaque: *mut c_void,
    ) -> *mut pxs_Var {
        unsafe {
            // Get ptr
            let pixel_object_var = pxs_Var::from_borrow(pxs_listget(args, 1));
            let host_ptr = pixel_object_var.get_host_ptr();
            let p = Person::from_borrow(host_ptr as *mut Person);

            pxs_Var::new_string(p.get_name().clone()).into_raw()
        }
    }

    pub extern "C" fn new_person(
        args: *mut pxs_Var,
        opaque: *mut c_void,
    ) -> *mut pxs_Var {
        unsafe {
            let p_name = pxs_Var::from_borrow(pxs_listget(args, 1));
            let p_name = p_name.get_string().unwrap();
            let p = Person::new(p_name.clone());
            let type_name = create_raw_string!("Person");

            let ptr = Person::into_raw(p) as *mut c_void;
            let pixel_object = pxs_newobject(ptr, free_person, type_name);
            let set_name_raw = create_raw_string!("set_name");
            let get_name_raw = create_raw_string!("get_name");
            pxs_object_addfunc(pixel_object, set_name_raw, set_name, opaque);
            pxs_object_addfunc(pixel_object, get_name_raw, get_name, opaque);
            // Save...
            let var = pxs_newhost(pixel_object);

            free_raw_string!(set_name_raw);
            free_raw_string!(get_name_raw);
            free_raw_string!(type_name);
            var
        }
    }

    // Testing callbacks
    pub extern "C" fn print_wrapper(
        args: *mut pxs_Var,
        _opaque: *mut c_void,
    ) -> *mut pxs_Var {
        unsafe {
            let var_ptr = pxs_Var::from_borrow(pxs_listget(args, 1));

            if let Ok(msg) = var_ptr.get_string() {
                println!("Lua sent: {}", msg);
            }
        }

        pxs_Var::new_null().into_raw()
    }

    pub extern "C" fn add_wrapper(
        args: *mut pxs_Var,
        _opaque: *mut c_void,
    ) -> *mut pxs_Var {
        // Assumes n1 and n2
        unsafe {
            let n1 = pxs_Var::from_borrow(pxs_listget(args, 1));
            let n2 = pxs_Var::from_borrow(pxs_listget(args, 2));

            pxs_Var::new_i64(n1.value.i64_val + n2.value.i64_val).into_raw()
        }
    }
    pub extern "C" fn sub_wrapper(
        args: *mut pxs_Var,
        _opaque: *mut c_void,
    ) -> *mut pxs_Var {
        // Assumes n1 and n2
        unsafe {
            let n1 = pxs_Var::from_borrow(pxs_listget(args, 1));
            let n2 = pxs_Var::from_borrow(pxs_listget(args, 2));

            pxs_Var::new_i64(n2.value.i64_val - n1.value.i64_val).into_raw()
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

    unsafe extern "C" fn call_function(
        args: pxs_VarT,
        _op: pxs_Opaque
    ) -> pxs_VarT {
        // Assume 1 is a function
        let func = pxs_listget(args, 1);
        // Check for args
        let argc = pxs_listlen(args);
        let res = if argc > 2 {
            // 2 is args
            pxs_varcall(pxs_listget(args, 0), func, pxs_newcopy(pxs_listget(args, 2)))
        } else {
            pxs_varcall(pxs_listget(args, 0), func, pxs_newlist())
        };

        // Return result!
        res
    }

    fn test_add_module() {
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
        let jordan = create_raw_string!("Jordan");
        let var = pxs_newstring(jordan);
        pxs_addvar(module, var_name, var);

        let object_name = create_raw_string!("Person");
        pxs_addobject(module, object_name, new_person, ptr::null_mut());

        // Add call 
        let call_name = create_raw_string!("call_function");
        pxs_addfunc(module, call_name, call_function, ptr::null_mut());
        free_raw_string!(call_name);

        // Add a inner module
        let math_module_name = create_raw_string!("math");
        let math_module = pxs_newmod(math_module_name);

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
        pxs_initialize();

        test_add_module();

        pxs_set_filereader(file_loader);

        let lua_code = r#"
            local pxs = require('pxs')
            local pxs_math = require('pxs.math')

            local ft_object = require('pad.ft_object')
            ft_object.function_from_outside()

            local msg = "Welcome, " .. pxs.name
            pxs.print(msg)

            local result = pxs.add(pxs.n1, pxs.n2)
            pxs.print(tostring(pxs.n1))
            pxs.print(tostring(pxs.n2))
            pxs.print(tostring(result))
            pxs.print("Module result: " .. tostring(result))

            if result ~= 3 then
                error("Math, Expected 3, got " .. tostring(result))
            end

            local res = pxs_math.sub(1, 2)

            if res ~= 1 then
                error("Math, Expected 1, got " .. tostring(res))
            end

            local person = pxs.Person("Jordan")
            pxs.print(person:get_name())
            person:set_name("Jordan Castro")
            pxs.print(person:get_name())

            -- Test calling function.
            function hadd(n1, n2)
                return n1 + n2
            end
            -- Call it
            pxs.print(tostring(pxs.call_function(hadd, {1,2})))
            function get_pi()
                return 3.145
            end 
            pxs.print(tostring(pxs.call_function(get_pi)))
        "#;
        let err = LuaScripting::execute(lua_code, "<test>");

        assert!(err.is_empty(), "Lua Error is not empty: {}", err);

        // Test eval
        let script = r#"
local pxs = require('pxs')
local pxs_math = require('pxs.math')
local pxs_math.sub = pxs_math.sub
function main()
    return pxs_math.sub(1,2)
end

return main()"#;
        let script_raw = create_raw_string!(script);

        let result = pxs_eval(script_raw, pxs_Runtime::pxs_Lua);
        assert!(!result.is_null(), "Lua result is null.");
        let str = pxs_tostring(pxs_newint(pxs_Runtime::pxs_Lua as i64), result);
        assert!(!str.is_null(), "Str is null.");
        let contents = pxs_getstring(str);
        assert!(!contents.is_null(), "Contents is null.");
        let owned = own_string!(contents);
        println!("{owned}");
        free_raw_string!(script_raw);

        pxs_finalize();
    }
}
