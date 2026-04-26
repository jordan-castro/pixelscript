// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_scope --no-default-features --features "lua,python,js,testing,pxs-debug" -- --nocapture --test-threads=1

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::{collections::HashMap, ffi::c_void};

    use pixelscript::{borrow_var, create_raw_string, free_raw_string, own_string, own_var, pxs_clearstate, pxs_compile, pxs_exec, pxs_execobject, pxs_finalize, pxs_freevar, pxs_gethost, pxs_getstring, pxs_initialize, pxs_listget, pxs_map_addpair, pxs_new_shallowcopy, pxs_newcopy, pxs_newfactory, pxs_newhost, pxs_newint, pxs_newlist, pxs_newmap, pxs_newnull, pxs_newobject, pxs_newstring, pxs_object_addfunc, pxs_startthread, pxs_stopthread, pxs_tostring, shared::{PtrMagic, pxs_Runtime, utils::setup_pxs, var::{pxs_Var, pxs_VarT}}};
    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    struct State {
        pub internals: HashMap<String, pxs_VarT>
    }

    impl PtrMagic for State {}
    impl Drop for State {
        fn drop(&mut self) {
            for (k, v) in self.internals.iter() {
                pxs_freevar(*v);
            }
        }
    }

    extern "C" fn free_state(ptr: *mut c_void) {
        let _ = unsafe{State::from_raw(ptr as *mut State)};
    }

    extern "C" fn get_attr(args: pxs_VarT) -> pxs_VarT {
        let obj = pxs_listget(args, 1);
        let state_ptr = pxs_gethost(pxs_listget(args, 0), obj);
        let state = unsafe{State::from_borrow_void(state_ptr)};

        let key = own_string!(pxs_getstring(pxs_listget(args, 2)));

        // Check if state contains arg...
        let item = state.internals.get(&key);
        if let Some(item) = item {
            pxs_new_shallowcopy(*item)
        } else {
            pxs_newnull()
        }
    }

    extern "C" fn set_attr(args: pxs_VarT) -> pxs_VarT {
        let obj = pxs_listget(args, 1);
        let state_ptr = pxs_gethost(pxs_listget(args, 0), obj);
        let state = unsafe{State::from_borrow_void(state_ptr)};

        let key = own_string!(pxs_getstring(pxs_listget(args, 2)));
        let value = pxs_newcopy(pxs_listget(args, 3));

        let old = state.internals.insert(key, value);
        if let Some(old) = old {
            pxs_freevar(old);
        }

        pxs_newnull()
    }

    extern "C" fn set_if_null(args: pxs_VarT) -> pxs_VarT {
        let obj = pxs_listget(args, 1);
        let state = unsafe{State::from_borrow_void(pxs_gethost(pxs_listget(args, 0), obj))};

        let key = own_string!(pxs_getstring(pxs_listget(args, 2)));
        let value = pxs_newcopy(pxs_listget(args, 3));

        if !state.internals.contains_key(&key) {
            state.internals.insert(key, value);
        }

        pxs_newnull()
    }

    extern "C" fn new_state(args: pxs_VarT) -> pxs_VarT {
        let state = State {
            internals: HashMap::new()
        };

        let type_name = create_raw_string!("State");
        let obj = pxs_newobject(state.into_raw() as *mut c_void, free_state, type_name);

        let getattr = create_raw_string!("get");
        let setattr = create_raw_string!("set");
        let setifnull = create_raw_string!("set_if_null");

        pxs_object_addfunc(obj, getattr, get_attr);
        pxs_object_addfunc(obj, setattr, set_attr);
        pxs_object_addfunc(obj, setifnull, set_if_null);

        unsafe { 
            free_raw_string!(type_name); 
            free_raw_string!(getattr);
            free_raw_string!(setattr);
            free_raw_string!(setifnull);
        };
        
        pxs_newhost(obj)
    } 


    fn scope() -> pxs_VarT {
        // Setup scope
        let scope = pxs_newmap();
        // New factory
        let state_args = pxs_newlist();
        let state_factory = new_state(state_args);
        // let state_factory = pxs_newfactory(new_state, pxs_newlist());
        let self_name = create_raw_string!("self");
        // Add state
        pxs_map_addpair(scope, pxs_newstring(self_name), state_factory);
        unsafe{
            free_raw_string!(self_name);
        }
        scope
    }

    fn test_python() {
        let code = r#"
import pxs

def init():
    self.set_if_null('name', "Jordan")
    self.set_if_null('age', 24)

init()
name = self.get('name')
age = self.get('age')
pxs.print(f"Hi! my name is {name} and I am {age} years old")
self.set('age', age + 1)

pxs.print(f'Current loop idx: {loop_id}')
"#;
        let raw_code = create_raw_string!(code);
        pxs_startthread();
        setup_pxs();
        // Compile python code to object
        let code_object = pxs_compile(pixelscript::shared::pxs_Runtime::pxs_Python, raw_code, scope());
        unsafe{
            free_raw_string!(raw_code);
        }

        // Print code object just to test
        let code_object_str = own_string!(pxs_getstring(pxs_tostring(pxs_newint(1), pxs_listget(code_object, 1))));
        println!("code object str: {code_object_str}");

        let loop_name = create_raw_string!("loop_id");  
        for i in 0..5 {
            let co = pxs_new_shallowcopy(code_object);
            let bco = borrow_var!(co);
            let local_scope = pxs_newmap();
            pxs_map_addpair(local_scope, pxs_newstring(loop_name), pxs_newint(i));
            // Run python code
            let res = own_var!(pxs_execobject(co, local_scope));
            assert!(res.is_null(), "Error found: {:#?}", res);
        }

        unsafe{free_raw_string!(loop_name);}
        pxs_freevar(code_object);
    }

    fn test_lua() {
        let code = r#"
local pxs = require('pxs')

function init()
    self:set_if_null('name', "Jordan")
    self:set_if_null('age', 24)
end
init()

local name = self:get('name')
local age = self:get('age')
pxs.print("Hi my name is " .. name .. " and I am " .. tostring(age) .. " years old")
self:set('age', age + 1)
pxs.print("Current loop idx: " .. tostring(loop_id))
"#;
        let raw_code = create_raw_string!(code);
        // Compile python code to object
        let code_object = pxs_compile(pixelscript::shared::pxs_Runtime::pxs_Lua, raw_code, scope());
        unsafe{
            free_raw_string!(raw_code);
        }

        let loop_name = create_raw_string!("loop_id");  
        for i in 0..5 {
            let co = pxs_new_shallowcopy(code_object);
            let bco = borrow_var!(co);
            let local_scope = pxs_newmap();
            pxs_map_addpair(local_scope, pxs_newstring(loop_name), pxs_newint(i));
            // Run lua code
            let res = own_var!(pxs_execobject(co, local_scope));
            assert!(res.is_null(), "Error found: {:#?}", res);
        }

        unsafe{free_raw_string!(loop_name);}
        pxs_freevar(code_object);

    }

    
    fn test_js() {
        let code = r#"
import * as pxs from 'pxs';

function init() {
    self.set_if_null('name', "Jordan");
    self.set_if_null('age', 24);
}

init();

let name = self.get('name');
let age = self.get('age');
pxs.print("Hi my name is " + name + " and I am " + age.toString() + " years old");
self.set('age', age + 1);
pxs.print("Current loop idx: " + loop_id.toString());
"#;
        let raw_code = create_raw_string!(code);
        // Compile python code to object
        let code_object = pxs_compile(pixelscript::shared::pxs_Runtime::pxs_JavaScript, raw_code, scope());
        unsafe{
            free_raw_string!(raw_code);
        }

        let co = borrow_var!(code_object);
        if co.is_exception() {
            println!("eXCEPTION dude: {:#?}", co);
        }

        let loop_name = create_raw_string!("loop_id");  
        for i in 0..5 {
            let co = pxs_new_shallowcopy(code_object);
            let bco = borrow_var!(co);
            let local_scope = pxs_newmap();
            pxs_map_addpair(local_scope, pxs_newstring(loop_name), pxs_newint(i));
            // Run lua code
            let res = own_var!(pxs_execobject(co, local_scope));
            assert!(res.is_null(), "Error found: {:#?}", res);
        }

        unsafe{free_raw_string!(loop_name);}
        pxs_freevar(code_object);

    }


    #[test]
    fn run_test() {
        pxs_initialize();

        pxs_startthread();
        // Setup module
        setup_pxs();
        pxs_clearstate(true);
        pxs_stopthread();
        pxs_startthread();

        print_helper("Python");
        test_python();
        print_helper("Lua");
        test_lua();
        print_helper("JavaScript");
        test_js();

        pxs_finalize();
    }
}
