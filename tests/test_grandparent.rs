// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_grandparent --no-default-features --features "lua,python,js,pxs-debug,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{
        pxs_add_submod, pxs_addfunc, pxs_addmod, pxs_arg, pxs_finalize, pxs_freearena, pxs_freestr, pxs_getrt, pxs_initialize, pxs_newarena, pxs_newmod, pxs_newnull, pxs_smart_getstring, shared::{module::pxs_Module, pxs_Runtime, utils, var::pxs_VarT},
    };
    use etffi::{cstring::CStringSafe, borrow_string, create_raw_string, free_raw_string, own_string, ptr_magic::PtrMagic};

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    extern "C" fn print_wrapper(args: pxs_VarT) -> pxs_VarT {
        let msg = own_string!(pxs_smart_getstring(pxs_getrt(args), pxs_arg(args, 0)));

        println!("{msg}");
        
        pxs_newnull()
    }

    fn test_python() {
        let script = r#"
from pxs.parent.child import *

print('Working Python')
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Python error is not null: {:#?}", res);
    }

    fn test_lua() {
        let script = r#"
local pxs = require('pxs.parent.child')

pxs.print('Working Lua')
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_null(), "Lua error is not null: {:#?}", res);
    }

    fn test_js() {
        let script = r#"
import * as pxs from 'pxs.parent.child';

pxs.print('Working JS');
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_JavaScript);
        assert!(res.is_null(), "JS error is not null: {:#?}", res);
    }

    #[test]
    fn run_test() {
        pxs_initialize();

        let parent = pxs_newmod(c"parent".as_ptr());
        let grandparent = pxs_newmod(c"pxs".as_ptr());
        let child = pxs_newmod(c"child".as_ptr());
        pxs_addfunc(child, c"print".as_ptr(), print_wrapper);
        pxs_add_submod(parent, child);
        pxs_add_submod(grandparent, parent);
        pxs_addmod(grandparent);

        print_helper("PYTHON");
        test_python();
        print_helper("LUA");
        test_lua();
        print_helper("JS");
        test_js();

        pxs_finalize();
    }
}
