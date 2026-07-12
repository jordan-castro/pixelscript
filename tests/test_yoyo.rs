// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_yoyo --no-default-features --features "lua,python,js,testing,yoyo_full" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{
        pxs_finalize, pxs_freearena, pxs_initialize, pxs_newarena, pxs_newmod, pxs_yoyoinit, shared::{module::pxs_Module, pxs_Runtime, utils, var::pxs_VarT},
    };
    use etffi::{cstring::CStringSafe, borrow_string, create_raw_string, free_raw_string, own_string, ptr_magic::PtrMagic};

    fn execute_yoyo(script: &str, rt: pxs_Runtime, test_name: &str) {
        let res = utils::execute_code(script, format!("<{test_name}>").as_str(), rt);
        assert!(res.is_null(), "Error executing {test_name}: {:#?}", res);
    }

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    fn test_core() {
        execute_yoyo("from yoyo import print, println\nprint('test 1 Python ')\nprintln('test 2 Python')", pxs_Runtime::pxs_Python, "core_py");
        execute_yoyo("local yoyo = require('yoyo') yoyo.print('test 1 Lua ') yoyo.println('test 2 Lua')", pxs_Runtime::pxs_Lua, "core_lua");
        execute_yoyo("import * as yoyo from 'yoyo'; yoyo.print('test 1 JS '); yoyo.println('test 2 JS');", pxs_Runtime::pxs_JavaScript, "core_js");
    }

    fn test_net() {
        execute_yoyo(include_str!("../core/yoyo/tests/net.py"), pxs_Runtime::pxs_Python, "net_py");
    }

    // fn test_net() {
    // }

    #[test]
    fn run_test() {
        println!();
        pxs_initialize();
        // utils::setup_pxs();
        pxs_yoyoinit();


        print_helper("core");
        test_core();
        print_helper("net");
        // test_net();
        // print_helper("net");
        // print_helper("fs");
        // print_helper("zip");

        pxs_finalize();
    }
}
