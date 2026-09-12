// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_exception_in_chunk --no-default-features --features "lua,python,js,pxs-debug,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{
        own_var, pxs_addmod, pxs_compile, pxs_execobject, pxs_finalize, pxs_freearena, pxs_initialize, pxs_new_shallowcopy, pxs_newarena, pxs_newexception, pxs_newmod, pxs_newnull, pxs_varis, shared::{module::pxs_Module, pxs_Runtime, utils, var::{pxs_VarT, pxs_VarType, pxs_Var}},
    };
    use etffi::{cstring::CStringSafe, borrow_string, create_raw_string, free_raw_string, own_string, ptr_magic::PtrMagic};

    extern "C" fn call(args: pxs_VarT) -> pxs_VarT {
        let msg = create_raw_string!("You no good dayo!!");
        let res = pxs_newexception(msg);
        unsafe { free_raw_string!(msg) };

        res
    }

    fn setup() {
        let module = utils::create_module("test_raise");
        utils::add_function(module, "call", call);
        pxs_addmod(module);
    }

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    fn compile_execute(script: &str, rt: pxs_Runtime, num: i32) {
        let mut cstring = CStringSafe::new();
        let co = pxs_compile(rt, cstring.new_string(script), pxs_newnull(), cstring.new_string("<test_exception_in_chunk>"));
        let res = pxs_execobject(pxs_new_shallowcopy(co), pxs_newnull());
        assert!(!res.is_null(), "Result is null");
        let reso = own_var!(res);
        assert!(reso.is_exception(), "Result is not exception: {:#?}", reso);

        println!("Exception: {num}");
    }

    fn test_python() {
        let script = r#"
import test_raise

test_raise.call()
"#;

        for i in 0..100 {
            compile_execute(script, pxs_Runtime::pxs_Python, i);
        }
    }

    fn test_lua() {
        let script = r#"
local pxs = require('test_raise')

test_raise.call()
"#;
        for i in 0..100 {
            compile_execute(script, pxs_Runtime::pxs_Lua, i);
        }

    }

    fn test_js() {
        let script = r#"
import * as test_raise from 'test_raise';

test_raise.call();
"#;
        for i in 0..100 {
            compile_execute(script, pxs_Runtime::pxs_Lua, i);
        }

    }

    #[test]
    fn run_test() {
        println!();
        pxs_initialize();
        utils::setup_pxs();
        setup();

        print_helper("PYTHON");
        test_python();
        print_helper("LUA");
        test_lua();
        print_helper("JS");
        test_js();

        pxs_finalize();
    }
}
