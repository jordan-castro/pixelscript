// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_arena --no-default-features --features "lua,python,js,pxs-debug,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{create_raw_string, free_raw_string, own_string, pxs_addfunc, pxs_addmod, pxs_addvar, pxs_clearstate, pxs_finalize, pxs_freearena, pxs_freevar, pxs_gethost, pxs_getint, pxs_getstring, pxs_initialize, pxs_listadd, pxs_listget, pxs_map_addpair, pxs_newarena, pxs_newbool, pxs_newfactory, pxs_newhost, pxs_newint, pxs_newlist, pxs_newmap, pxs_newmod, pxs_newnull, pxs_newobject, pxs_newstring, shared::{PtrMagic, module::pxs_Module, pxs_Opaque, pxs_Runtime, utils::{self, CStringSafe}, var::pxs_VarT}};

    struct Person2 {
        person: Person
    }

    impl PtrMagic for Person2 {}
    extern "C" fn drop_person2(ptr: pxs_Opaque) {
        let _ = Person2::from_raw(ptr as *mut Person2);
    }
    extern "C" fn new_person2(args: pxs_VarT) -> pxs_VarT {
        let p = unsafe{Person::from_borrow_void(pxs_gethost(pxs_listget(args, 0), pxs_listget(args, 1)))};
        let p2 = Person2{
            person: Person{
                name: p.name.clone()
            }
        };

        let mut cstrgen = CStringSafe::new();
        let obj = pxs_newobject(p2.into_void(), drop_person2, cstrgen.new_string("Person2"));
        pxs_newhost(obj)
    }

    struct Person {
        name: String
    }

    impl PtrMagic for Person {}

    extern "C" fn drop_person(ptr: pxs_Opaque) {
        let _ = Person::from_raw(ptr as *mut Person);
    }

    extern "C" fn new_person(args: pxs_VarT) -> pxs_VarT {
        let name = own_string!(pxs_getstring(pxs_listget(args, 1)));
        let person = Person{name};
        let mut cstrgen = CStringSafe::new();
        let obj = pxs_newobject(person.into_void(), drop_person, cstrgen.new_string("Person"));
        pxs_newhost(obj)
    }

    extern "C" fn allocate_memory(args: pxs_VarT) -> pxs_VarT {
        // Just allocate memory based on num
        let num = pxs_getint(pxs_listget(args, 1));

        let mut cstrgen = CStringSafe::new();
        // What what what?? We are not freeing anything!
        for i in 0..num {
            pxs_newnull();
            // pxs_newint(3);
            let list = pxs_newlist();
            pxs_listadd(list, pxs_newint(0));
            let map = pxs_newmap();
            pxs_map_addpair(map, pxs_newint(12), pxs_newbool(false));

            // This gets freed
            let list2 = pxs_newlist();
            let list3 = pxs_newlist();
            pxs_listadd(list3, pxs_newint(10));
            pxs_listadd(list2, list3);

            let f1 = pxs_newfactory(new_person, args);
            let f2_args = pxs_newlist();
            pxs_listadd(f2_args, f1);
            let f2 = pxs_newfactory(new_person2, f2_args);

            pxs_freevar(list2);
        }

        pxs_newbool(true)
    }

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    fn test_python() {
        let script = r#"
from test import alloc
alloc(5)
"#;
        pxs_newarena();
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Python error is not null: {:#?}", res);
        pxs_freearena();
    }

    fn test_lua() {
        let script = r#"
local alloc = require('test').alloc

alloc(15)
"#;
        pxs_newarena();
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_null(), "Lua error is not null: {:#?}", res);
        pxs_freearena();
    }

    fn test_js() {
        let script = r#"
// import * as pxs from 'pxs';
// pxs.print('test');
import {alloc} from 'test';

alloc(20);
"#;
        pxs_newarena();
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_JavaScript);
        assert!(res.is_null(), "JS error is not null: {:#?}", res);
        pxs_freearena();
    }

    #[test]
    fn run_test() {
        println!();
        pxs_initialize();
        utils::setup_pxs();

        pxs_newarena();
        let mut cstrgen = CStringSafe::new();
        let test_mod = pxs_newmod(cstrgen.new_string("test"));
        pxs_addfunc(test_mod, cstrgen.new_string("alloc"), allocate_memory);
        
        // Testing factory
        let factory_args = pxs_newlist();
        pxs_listadd(factory_args, pxs_newstring(cstrgen.new_string("contents")));
        let person_factory = pxs_newfactory(new_person, factory_args);
        pxs_addvar(test_mod, cstrgen.new_string("person"), person_factory);
        pxs_addmod(test_mod);

        print_helper("PYTHON");
        test_python();
        print_helper("LUA");
        test_lua();
        print_helper("JS");
        test_js();

        pxs_clearstate(true);
        pxs_freearena();
        pxs_finalize();
    }
}
