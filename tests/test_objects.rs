// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_objects --no-default-features --features "lua,python,testing,include-core" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use std::ffi::c_void; 

    use pixelscript::{borrow_var, create_raw_string, free_raw_string, own_string, pxs_addmod, pxs_addobject, pxs_finalize, pxs_gethost, pxs_getint, pxs_getstring, pxs_getuint, pxs_initialize, pxs_listadd, pxs_listget, pxs_listlen, pxs_newhost, pxs_newint, pxs_newlist, pxs_newmod, pxs_newnull, pxs_newobject, pxs_newstring, pxs_newuint, pxs_object_addfunc, pxs_object_addprop, shared::{PtrMagic, module::pxs_Module, pxs_Runtime, utils::{self, CStringSafe}, var::{pxs_Var, pxs_VarT}}};
    
    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    #[derive(Clone, Debug)]
    struct Person {
        name: String,
        age: u32
    }
    impl PtrMagic for Person {}

    extern "C" fn free_person(ptr: *mut c_void) {
        let _ = Person::from_raw(ptr as *mut Person);
    }
    
    extern "C" fn person_name_prop(args: pxs_VarT) -> pxs_VarT {
        let p = unsafe { Person::from_borrow_void(pxs_gethost(pxs_listget(args, 0), pxs_listget(args, 1))) };
        if pxs_listlen(args) == 2 {
            let mut cstrgen = CStringSafe::new();
            pxs_newstring(cstrgen.new_string(&p.name))
        } else {
            // Get value
            let new_name = own_string!(pxs_getstring(pxs_listget(args, 2)));
            p.name = new_name;
            pxs_newnull()
        }
    }

    extern "C" fn person_age_prop(args: pxs_VarT) -> pxs_VarT {
        let p = unsafe { Person::from_borrow_void(pxs_gethost(pxs_listget(args, 0), pxs_listget(args, 1))) };
        if pxs_listlen(args) == 2 {
            pxs_newuint(p.age as u64)
        } else {
            // Get value
            let new_age = pxs_getuint(pxs_listget(args, 2));
            p.age = new_age as u32;
            pxs_newnull()
        }
    }

    extern "C" fn new_person(args: pxs_VarT) -> pxs_VarT {
        let name = own_string!(pxs_getstring(pxs_listget(args, 1)));
        let age = pxs_getint(pxs_listget(args, 2));

        let person = Person{name, age: age as u32};
        let mut cstrgen = CStringSafe::new();
        let tname = cstrgen.new_string("Person");
        let obj = pxs_newobject(person.into_void(), free_person, tname);

        pxs_object_addprop(obj, cstrgen.new_string("name"), person_name_prop);
        pxs_object_addprop(obj, cstrgen.new_string("age"), person_age_prop);

        pxs_newhost(obj)
    }

    struct Diary {
        owner: Person,
        entries: Vec<String>
    }
    impl PtrMagic for Diary {}

    extern "C" fn free_diary(ptr: *mut c_void) {
        let _ = Diary::from_raw(ptr as *mut Diary);
    }

    extern "C" fn diary_owner_prop(args: pxs_VarT) -> pxs_VarT {
        let diary = unsafe{Diary::from_borrow_void(pxs_gethost(pxs_listget(args, 0), pxs_listget(args, 1)))};
        if pxs_listlen(args) == 2 {
            let owner = &diary.owner;
            let nargs = pxs_newlist();
            let mut cstrgen = CStringSafe::new();
            pxs_listadd(nargs, pxs_listget(args, 0));
            pxs_listadd(nargs, pxs_newstring(cstrgen.new_string(&owner.name)));
            pxs_listadd(nargs, pxs_newint(owner.age as i64));
            new_person(nargs)
        } else {
            // Set new owner...
            let owner = unsafe{Person::from_borrow_void(pxs_gethost(pxs_listget(args, 0), pxs_listget(args, 2)))};
            diary.owner = owner.clone();
            pxs_newnull()
        }
    }

    extern "C" fn diary_entries_prop(args: pxs_VarT) -> pxs_VarT {
        let diary = unsafe{Diary::from_borrow_void(pxs_gethost(pxs_listget(args, 0), pxs_listget(args, 1)))};
        if pxs_listlen(args) == 2 {
            let list = pxs_newlist();
            let mut cstrgen = CStringSafe::new();
            for i in diary.entries.iter() {
                pxs_listadd(list, pxs_newstring(cstrgen.new_string(i)));
            }
            list
        } else {
            // println!("{:#?}", borrow_var!(pxs_listget(args, 2)));
            // Update entries?
            let list = pxs_listget(args, 2);
            let mut items = vec![];
            for i in 0..pxs_listlen(list) {
                items.push(own_string!(pxs_getstring(pxs_listget(list, i))));
            }
            println!("items: {:#?}", items);
            diary.entries = items;
            pxs_newnull()
        }
    }

    extern "C" fn diary_string(args: pxs_VarT) -> pxs_VarT {
        let diary = unsafe{Diary::from_borrow_void(pxs_gethost(pxs_listget(args, 0), pxs_listget(args, 1)))};
        let mut cstrgen = CStringSafe::new();
        let string = format!("{:#?}, {:#?}", diary.owner, diary.entries);
        pxs_newstring(cstrgen.new_string(&string))
    }

    extern "C" fn new_diary(args: pxs_VarT) -> pxs_VarT {
        let owner = unsafe{Person::from_borrow_void(pxs_gethost(pxs_listget(args, 0), pxs_listget(args, 1)))};

        let diary = Diary{owner: owner.clone(), entries: vec![]};
        let mut cstrgen = CStringSafe::new();
        let tname = cstrgen.new_string("Diary");
        let obj = pxs_newobject(diary.into_void(), free_diary, tname);

        pxs_object_addprop(obj, cstrgen.new_string("owner"), diary_owner_prop);
        pxs_object_addprop(obj, cstrgen.new_string("entries"), diary_entries_prop);
        pxs_object_addfunc(obj, cstrgen.new_string("__str__"), diary_string);
        pxs_object_addfunc(obj, cstrgen.new_string("__tostring"), diary_string);

        pxs_newhost(obj)
    }

    fn test_python() {
        let script = r#"
from pxs import *
from test import *

p = Person('Jordan', 24)
d = Diary(p)
d.entries += ["Test dude"]
d.entries += ["Another one"]
d.entries += ["Dude did I just make this work?"]

print(d.entries)
print(p.name)
p.age += 1
print(p.age)
"#;

        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Error: {:#?}", res);
    }
    fn test_lua() {
        let script = r#"
local pxs  = require('pxs')
local test = require('test')

local p = test.Person('Jordan', 24)
local d = test.Diary(p)
d.entries = {'test'}
local e = d.entries
e[2] = 'test 2'
d.entries = e
pxs.print(pxs_json.encode(d.entries))

-- d.entries = d.entries + {'Test dude'}
-- d.entries = d.entries + {'Another one'}
-- d.entries = d.entries + {'Dude did I just make this work?'}

pxs.print(tostring(d))
pxs.print(p.name)
p.age = p.age + 1
pxs.print(p.age)

"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_null(), "Error: {:#?}", res);
    }

    #[test]
    fn run_test() {
        pxs_initialize();
        utils::setup_pxs();

        let mut cgen = CStringSafe::new();
        let module = pxs_newmod(cgen.new_string("test"));
        pxs_addobject(module, cgen.new_string("Person"), new_person);
        pxs_addobject(module, cgen.new_string("Diary"), new_diary);
        pxs_addmod(module);

        print_helper("PYTHON");
        test_python();
        print_helper("LUA");
        test_lua();

        pxs_finalize();
    }
}
