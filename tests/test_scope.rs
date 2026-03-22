// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_scope --no-default-features --features "lua,python,pxs-debug,testing" -- --nocapture --test-threads=1

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::{collections::HashMap, ffi::c_void};

    use pixelscript::{create_raw_string, free_raw_string, own_string, own_var, pxs_compile, pxs_execobject, pxs_finalize, pxs_freevar, pxs_gethost, pxs_getstring, pxs_initialize, pxs_listget, pxs_map_addpair, pxs_new_shallowcopy, pxs_newcopy, pxs_newfactory, pxs_newhost, pxs_newlist, pxs_newmap, pxs_newnull, pxs_newobject, pxs_newstring, pxs_object_addfunc, shared::{PtrMagic, utils::setup_pxs, var::{pxs_Var, pxs_VarT}}};

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
        let _ = unsafe{State::from_borrow_void(ptr)};
    }

    extern "C" fn get_attr(args: pxs_VarT) -> pxs_VarT {
        let obj = pxs_listget(args, 1);
        let state_ptr = pxs_gethost(pxs_listget(args, 0), obj);
        let state = unsafe{State::from_borrow_void(state_ptr)};

        let key = own_string!(pxs_getstring(pxs_listget(args, 1)));

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

        let key = own_string!(pxs_getstring(pxs_listget(args, 1)));
        let value = pxs_newcopy(pxs_listget(args, 2));

        let old = state.internals.insert(key, value);
        if let Some(old) = old {
            pxs_freevar(old);
        }

        pxs_newnull()
    }

    extern "C" fn new_state(args: pxs_VarT) -> pxs_VarT {
        let state = State {
            internals: HashMap::new()
        };

        let type_name = create_raw_string!("State");
        let obj = pxs_newobject(state.into_raw() as *mut c_void, free_state, type_name);

        let getattr = create_raw_string!("__getattr__");
        let setattr = create_raw_string!("__setattr__");
        let index = create_raw_string!("__index");
        let newindex = create_raw_string!("__newindex");

        pxs_object_addfunc(obj, getattr, get_attr);
        pxs_object_addfunc(obj, setattr, set_attr);
        pxs_object_addfunc(obj, index, get_attr);
        pxs_object_addfunc(obj, newindex, set_attr);

        unsafe { 
            free_raw_string!(type_name); 
            free_raw_string!(getattr);
            free_raw_string!(setattr);
            free_raw_string!(index);
            free_raw_string!(newindex);
        };
        
        pxs_newhost(obj)
    } 

    fn test_python() {
        let code = r#"
import pxs

def init():
    if not self.name:
        self.name = "Jordan"
    if not self.age:
        self.age = 24

init()
pxs.print(f"Hi! my name is {self.name} and I am {self.age} years old")
self.age += 1
"#;

        // Setup scope
        let scope = pxs_newmap();
        // New factory
        let state_factory = pxs_newfactory(new_state, pxs_newlist());
        let self_name = create_raw_string!("self");
        // Add state
        pxs_map_addpair(scope, pxs_newstring(self_name), state_factory);

        let raw_code = create_raw_string!(code);
        // Compile python code to object
        let code_object = pxs_compile(pixelscript::shared::pxs_Runtime::pxs_Python, raw_code, scope);
        unsafe{
            free_raw_string!(raw_code);
            free_raw_string!(self_name);
        }

        for i in 0..5 {
            // Run python code
            let res = own_var!(pxs_execobject(code_object));
            assert!(res.is_null(), "Error found: {}", res.get_string().unwrap());
        }
    }
    fn test_lua() {
        let code = r#"
local pxs = require('pxs')

function init()
    
end
"#;
    }

    #[test]
    fn run_test() {
        pxs_initialize();

        // Setup module
        setup_pxs();

        test_python();
        test_lua();

        pxs_finalize();
    }
}
