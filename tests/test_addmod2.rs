// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_addmod2 --no-default-features --features "lua,python,js,pxs-debug,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{
        pxs_addfunc, pxs_addmod, pxs_addmod2, pxs_finalize, pxs_freearena, pxs_freemod, pxs_initialize, pxs_newarena, pxs_newint, pxs_newmod, shared::{module::pxs_Module, pxs_Runtime, utils, var::pxs_VarT},
    };
    use etffi::{cstring::CStringSafe, borrow_string, create_raw_string, free_raw_string, own_string, ptr_magic::PtrMagic};

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    fn test_python() {
        let script = r#"
import all
import py

py.add()
all.add()
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Python error is not null: {:#?}", res);

        let script = "import js\njs.add()";
        let res = utils::execute_code(script, "<fail_test>", pxs_Runtime::pxs_Python);
        assert!(res.is_exception(), "Python error is null: {:#?}", res);
    }

    fn test_lua() {
        let script = r#"
local all = require('all')
local lua = require('lua')

lua.add()
all.add()
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_null(), "Lua error is not null: {:#?}", res);

        let script = "local py = require('py')\npy.add()";
        let res = utils::execute_code(script, "<fail_test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_exception(), "Lua error is null: {:#?}", res);
    }

    fn test_js() {
        let script = r#"
import * as js from 'js';
import * as all from 'all';

js.add();
all.add();
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_JavaScript);
        assert!(res.is_null(), "JS error is not null: {:#?}", res);

        let script = "import * as lua from 'lua';\nlua.add();";
        let res = utils::execute_code(script, "<fail_test>", pxs_Runtime::pxs_JavaScript);
        assert!(res.is_exception(), "JS error is null: {:#?}", res);
    }

    extern "C" fn py_add(args: pxs_VarT) -> pxs_VarT {
        println!("py add!");
        pxs_newint(1)
    }

    extern "C" fn lua_add(args: pxs_VarT) -> pxs_VarT {
        println!("lua add!");
        pxs_newint(2)
    }

    extern "C" fn js_add(args: pxs_VarT) -> pxs_VarT {
        println!("js add");
        pxs_newint(3)
    }

    extern "C" fn all_add(args: pxs_VarT) -> pxs_VarT {
        println!("all add");
        pxs_newint(4)
    }

    #[test]
    fn run_test() {
        println!();
        pxs_initialize();

        let py_module = pxs_newmod(c"py".as_ptr());
        let lua_module = pxs_newmod(c"lua".as_ptr());
        let js_module = pxs_newmod(c"js".as_ptr());

        let all_module = pxs_newmod(c"all".as_ptr());

        pxs_addfunc(py_module, c"add".as_ptr(), py_add);
        pxs_addfunc(lua_module, c"add".as_ptr(), lua_add);
        pxs_addfunc(js_module, c"add".as_ptr(), js_add);
        pxs_addfunc(all_module, c"add".as_ptr(), all_add);

        pxs_addmod2(lua_module, pxs_Runtime::pxs_Lua as u8);
        pxs_addmod2(py_module, pxs_Runtime::pxs_Python as u8);
        pxs_addmod2(js_module, pxs_Runtime::pxs_JavaScript as u8);
        pxs_addmod(all_module);

        pxs_freemod(lua_module);
        pxs_freemod(py_module);
        pxs_freemod(js_module);
        // pxs_addmod2(all_module, ((pxs_Runtime::pxs_Lua as u8) << 0) | ((pxs_Runtime::pxs_Python as u8) << 1) | ((pxs_Runtime::pxs_JavaScript as u8) << 2));

        print_helper("PYTHON");
        test_python();
        print_helper("LUA");
        test_lua();
        print_helper("JS");
        test_js();

        pxs_finalize();
    }
}
