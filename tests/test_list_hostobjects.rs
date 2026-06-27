// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_list_hostobjects --no-default-features --features "python,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{
        pxs_addfunc, pxs_addmod, pxs_finalize, pxs_freearena,
        pxs_freevar, pxs_getint, pxs_initialize, pxs_listadd, pxs_listget, pxs_newarena,
        pxs_newhost, pxs_newint, pxs_newlist, pxs_newmod, pxs_newobject, pxs_newstring,
        shared::{
            module::pxs_Module,
            pxs_Opaque, pxs_Runtime,
            utils::{self},
            var::pxs_VarT,
        },
    };
    use etffi::{cstring::CStringSafe, borrow_string, create_raw_string, free_raw_string, own_string, ptr_magic::PtrMagic};

    struct Person {
        age: i32,
    }

    impl PtrMagic for Person {}

    extern "C" fn free_person(ptr: pxs_Opaque) {
        unsafe {
            let _ = Person::from_raw(ptr as *mut Person);
        }
    }

    extern "C" fn new_person(args: pxs_VarT) -> pxs_VarT {
        let person = Person { age: 1 };
        let mut cstrgen = CStringSafe::new();
        let pixel_object = pxs_newobject(
            person.into_void(),
            free_person,
            cstrgen.new_string("Person"),
        );

        pxs_newhost(pixel_object)
    }

    extern "C" fn host_persons(args: pxs_VarT) -> pxs_VarT {
        let num = pxs_getint(pxs_listget(args, 1));
        let list = pxs_newlist();

        let mut cstrgen = CStringSafe::new();
        for i in 0..num {
            let args = pxs_newlist();
            let person = unsafe { new_person(args) };
            pxs_freevar(args);
            pxs_listadd(list, person);
        }
        list
    }

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    fn test_python() {
        let script = r#"
from pxs import *
from ho import host_persons

result = host_persons(10)

for item in result:
    print(item)

#if not type(result) is list:
#    raise Exception('NOT 10, ' + str(result))

print('Working Python')
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Python error is not null: {:#?}", res);
    }

    #[test]
    fn run_test() {
        println!();
        pxs_initialize();
        utils::setup_pxs();

        let mut cstrgen = CStringSafe::new();
        let ho_mod = pxs_newmod(cstrgen.new_string("ho"));
        pxs_addfunc(ho_mod, cstrgen.new_string("host_persons"), host_persons);
        pxs_addmod(ho_mod);

        test_python();

        pxs_finalize();
    }
}
