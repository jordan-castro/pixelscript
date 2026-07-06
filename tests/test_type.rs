// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_type --no-default-features --features "lua,python,js,pxs-debug,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{
        pxs_addfunc, pxs_addmod, pxs_addobject, pxs_finalize, pxs_freearena, pxs_getint, pxs_gettype, pxs_initialize, pxs_listget, pxs_newarena, pxs_newhost, pxs_newint, pxs_newmod, pxs_newtype, shared::{module::pxs_Module, pxs_Opaque, pxs_Runtime, utils, var::{pxs_Var, pxs_VarT}},
    };
    use etffi::{cstring::CStringSafe, borrow_string, create_raw_string, free_raw_string, own_string, ptr_magic::PtrMagic};

    struct Vector2 {
        x:i64,
        y:i64
    }

    impl PtrMagic for Vector2 {}

    unsafe extern "C" fn free_vector2(obj: pxs_Opaque) {
        Vector2::from_raw(obj as *mut Vector2);
    }

    unsafe extern "C" fn new_vector2(args: pxs_VarT) -> pxs_VarT {
        let x = pxs_getint(pxs_listget(args, 1));
        let y = pxs_getint(pxs_listget(args, 2));

        let mut cstring = CStringSafe::new();
        let v2 = Vector2{x,y};
        let t = pxs_newtype(v2.into_void(), free_vector2, cstring.new_string("Vector2"), 0);
        pxs_newhost(t)
    }

    unsafe extern "C" fn add_vectors_x(args: pxs_VarT) -> pxs_VarT {
        let rt = pxs_listget(args, 0);
        let v1 = pxs_gettype(rt, pxs_listget(args, 1), 0);
        let v2 = pxs_gettype(rt, pxs_listget(args, 2), 0);
        if v1.is_null() {
            return pxs_Var::new_exception("V1 is not of type Vector").into_raw();
        }
        if v2.is_null() {
            return pxs_Var::new_exception("V2 is not of type Vector").into_raw();
        }

        let x = unsafe { Vector2::from_borrow_void(v1).x + Vector2::from_borrow_void(v2).x };
        pxs_newint(x)
    }

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    fn test_python() {
        let script = r#"
import pxs
import test_types as tt

v1 = tt.Vector2(0,1)
v2 = tt.Vector2(1,0)

xs = tt.add_vx(v1, v2)
if xs != 1:
    raise Exception("xs not equal to 1")
pxs.print("Working Python!")
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Python error is not null: {:#?}", res);
    }

    fn test_lua() {
        let script = r#"
local pxs = require('pxs')
local tt = require("test_types")

local v1 = tt.Vector2(0,1)
local v2 = tt.Vector2(1,0)

local xs = tt.add_vx(v1, v2)
if xs ~= 1 then
    error("xs not equal to 1")
end

pxs.print('Working Lua')
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_null(), "Lua error is not null: {:#?}", res);
    }

    fn test_js() {
        let script = r#"
import * as pxs from 'pxs';
import * as tt from 'test_types';

let v1 = tt.Vector2(0,1);
let v2 = tt.Vector2(1,0);

let xs = tt.add_vx(v1, v2);
if (xs != 1) {
    throw 'xs not equal to 1';
}

pxs.print('Working JS');
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_JavaScript);
        assert!(res.is_null(), "JS error is not null: {:#?}", res);
    }

    #[test]
    fn run_test() {
        println!();
        pxs_initialize();


        let module = pxs_newmod(c"test_types".as_ptr());
        pxs_addobject(module, c"Vector2".as_ptr(), new_vector2);
        pxs_addfunc(module, c"add_vx".as_ptr(), add_vectors_x);
        pxs_addmod(module);

        utils::setup_pxs();

        let test_mod = pxs_Module::new("name".to_string());

        print_helper("PYTHON");
        test_python();
        print_helper("LUA");
        test_lua();
        print_helper("JS");
        test_js();

        pxs_finalize();
    }
}
