// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_refcount --no-default-features --features "lua,python,js,pxs-debug,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use std::ffi::c_void;

    use pixelscript::{create_raw_string, free_raw_string, own_string, own_var, pxs_addmod, pxs_addobject, pxs_finalize, pxs_gethost, pxs_getstring, pxs_initialize, pxs_listget, pxs_newhost, pxs_newmod, pxs_newobject, pxs_newstring, pxs_object_addfunc, shared::{PtrMagic, module::pxs_Module, pxs_Runtime, utils, var::pxs_VarT}};
    
    fn print_helper(lang: &str) {
        print!("====================== {lang} ===================");
    }

    struct Person {
        name: String
    }

    impl PtrMagic for Person {}

    extern "C" fn free_person(ptr: *mut c_void) {
        println!("Freeing person!: {:p}", ptr);
        let _ = Person::from_raw(ptr as *mut Person);
    }

    extern "C" fn new_person(args: pxs_VarT) -> pxs_VarT {
        let name = pxs_listget(args, 1);
        let name_str = own_string!(pxs_getstring(name));

        let person = Person{name: name_str};
        let person_name = create_raw_string!("Person");
        let object = pxs_newobject(person.into_raw() as *mut c_void, free_person, person_name);
        unsafe{
            free_raw_string!(person_name);
        }

        let name = create_raw_string!("get_name");
        pxs_object_addfunc(object, name, get_name);
        unsafe{
            free_raw_string!(name);
        }

        pxs_newhost(object)
    }

    extern "C" fn get_name(args: pxs_VarT) -> pxs_VarT {
        let person_ptr = pxs_listget(args, 1);
        let person = unsafe { Person::from_borrow_void(pxs_gethost(pxs_listget(args, 0), person_ptr)) };

        let result = create_raw_string!(person.name.clone());
        let var = pxs_newstring(result);
        unsafe{
            free_raw_string!(result);
        }

        var
    }

    fn test_python() {
        let py_script = r#"
from test import Person
from pxs import *

p = Person('Jordan')
print(p)
print(p.get_name())
"#;
        let res = utils::execute_code(py_script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Py ERror is not null: {:#?}", res);
    }
    fn test_lua() {
        let lua_script = r#"
local test = require('test')
local pxs = require('pxs')

local p = test.Person('Jordan')
pxs.print(p)
pxs.print(p:get_name())
"#;
        let res = utils::execute_code(lua_script, "<test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_null(), "lua error is not null: {:#?}", res);
    }

    fn test_js() {
        let js_script = r#"
import {Person} from 'test';
import {print} from 'pxs';

let p = Person('Jordan');
print(p)
print(p.get_name())
"#;
        let res = utils::execute_code(js_script, "<test>", pxs_Runtime::pxs_JavaScript);
        assert!(res.is_null(), "lua error is not null: {:#?}", res);
    }

    #[test]
    fn run_test() {
        pxs_initialize();
        utils::setup_pxs();
        let test_module = utils::create_module("test");
        let name = create_raw_string!("Person");
        pxs_addobject(test_module, name, new_person);
        unsafe {
            free_raw_string!(name);
        }
        pxs_addmod(test_module);

        print_helper("PYTHON");
        test_python();
        print_helper("LUA");
        test_lua();
        print_helper("JavaScript");
        test_js();
        
        pxs_finalize();
    }
}
