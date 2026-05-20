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
    use pixelscript::{create_raw_string, free_raw_string, own_string, pxs_addfunc, pxs_addmod, pxs_addvar, pxs_arenaput, pxs_clearstate, pxs_finalize, pxs_freearena, pxs_freevar, pxs_gethost, pxs_getint, pxs_getstring, pxs_initialize, pxs_listadd, pxs_listget, pxs_map_addpair, pxs_newarena, pxs_newbool, pxs_newfactory, pxs_newhost, pxs_newint, pxs_newlist, pxs_newmap, pxs_newmod, pxs_newnull, pxs_newobject, pxs_newstring, shared::{PtrMagic, module::pxs_Module, pxs_Opaque, pxs_Runtime, utils::{self, CStringSafe}, var::pxs_VarT}};

    struct Person2 {
        person: Person
    }

    impl PtrMagic for Person2 {}
    extern "C" fn drop_person2(ptr: pxs_Opaque) {
        let _ = Person2::from_raw(ptr as *mut Person2);
    }
    extern "C" fn new_person2(args: pxs_VarT) -> pxs_VarT {
        println!("new_person_2");
        let p = unsafe{Person::from_borrow_void(pxs_gethost(pxs_listget(args, 0), pxs_listget(args, 1)))};
        println!("person2 name: {}", p.name);
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
        println!("Person");
        let name = own_string!(pxs_getstring(pxs_listget(args, 1)));
        println!("Name: {name}");
        let person = Person{name};
        let mut cstrgen = CStringSafe::new();
        let obj = pxs_newobject(person.into_void(), drop_person, cstrgen.new_string("Person"));
        pxs_newhost(obj)
    }

    extern "C" fn allocate_memory(args: pxs_VarT) -> pxs_VarT {
        // Just allocate memory based on num
        let num = pxs_getint(pxs_listget(args, 1));

        let arena = pxs_newarena();

        let mut cstrgen = CStringSafe::new();
        // What what what?? We are not freeing anything!
        for i in 0..num {
            pxs_arenaput(arena, pxs_newnull());
            // pxs_newint(3);
            let list = pxs_newlist();
            pxs_arenaput(arena, list);
            pxs_listadd(list, pxs_newint(0));
            let map = pxs_arenaput(arena, pxs_newmap());
            pxs_map_addpair(map, pxs_newint(12), pxs_newbool(false));

            // This gets freed
            let list2 = pxs_arenaput(arena, pxs_newlist());
            let list3 = pxs_newlist();
            pxs_listadd(list3, pxs_newint(10));
            pxs_listadd(list2, list3);

            // The reason it crashed was because... f1 becomes owned by f2_args
            // and it was being passed to the arena as if it owned it.
            // Be very careful with who owns what pointer. I might add a new
            // feature flag of (strict) to make it impossible to pass a owned pointer
            // to any function that transfers memory
            let person_args = pxs_newlist();
            pxs_listadd(person_args, pxs_newstring(cstrgen.new_string("Jordan")));
            let f1 = pxs_newfactory(new_person, person_args);
            let f2_args = pxs_newlist();
            pxs_listadd(f2_args, f1);
            let f2 = pxs_arenaput(arena, pxs_newfactory(new_person2, f2_args));
        }

        pxs_freearena(arena);
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
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Python error is not null: {:#?}", res);
    }

    fn test_lua() {
        let script = r#"
local alloc = require('test').alloc

alloc(15)
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_null(), "Lua error is not null: {:#?}", res);
    }

    fn test_js() {
        let script = r#"
// import * as pxs from 'pxs';
// pxs.print('test');
import {alloc} from 'test';

alloc(20);
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_JavaScript);
        assert!(res.is_null(), "JS error is not null: {:#?}", res);
    }

    #[test]
    fn run_test() {
        println!();
        pxs_initialize();
        utils::setup_pxs();

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
        pxs_finalize();
    }
}
