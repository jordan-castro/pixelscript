// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_eval --no-default-features --features "lua,python,js,pxs-debug,testing,js_commonjs" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{
        pxs_finalize, pxs_freearena, pxs_initialize,
        pxs_newarena, pxs_newmod,
        shared::{module::pxs_Module, pxs_Runtime, utils, var::pxs_VarT},
    };
    use etffi::{cstring::CStringSafe, borrow_string, create_raw_string, free_raw_string, own_string, ptr_magic::PtrMagic};

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    fn test_python() {
        let script = r#"1 + 1"#;
        let res = utils::eval_code(script, "<py>", pxs_Runtime::pxs_Python);
        assert!(res.is_i64(), "Python error is not i64: {:#?}", res);
        assert!(res.get_i64().unwrap() == 2, "Python res is not 2");
    }

    fn test_lua() {
        let script = "return 1 + 1";
        let res = utils::eval_code(script, "<lua>", pxs_Runtime::pxs_Lua);
        assert!(res.is_i64(), "Lua error is not i64: {:#?}", res);
        assert!(res.get_i64().unwrap() == 2, "Lua res is not 2");
    }

    fn test_js() {
        let script = r#"
        const pxs = require("pxs");
        pxs.print('test from JS');
        1 + 1;
"#;
        let res = utils::eval_code(script, "<js>", pxs_Runtime::pxs_JavaScript);
        assert!(res.is_i64(), "JS error is not i64: {:#?}", res);
        assert!(res.get_i64().unwrap() == 2, "JS res is not 2");
    }

    #[test]
    fn run_test() {
        println!();
        pxs_initialize();
        utils::setup_pxs();

        print_helper("PYTHON");
        test_python();
        print_helper("LUA");
        test_lua();
        print_helper("JS");
        test_js();

        pxs_finalize();
    }
}
