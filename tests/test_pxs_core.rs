// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_pxs_core --no-default-features --features "lua,python,js,testing,include-core" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{
        pxs_core_initall, pxs_finalize, pxs_freearena, pxs_initialize, pxs_newarena, pxs_newmod, shared::{module::pxs_Module, pxs_Runtime, utils, var::pxs_VarT},
    };
    use etffi::{cstring::CStringSafe, borrow_string, create_raw_string, free_raw_string, own_string, ptr_magic::PtrMagic};

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    fn test_python() {
        let script = r#"
import pxs
import pxs_fs
import pxs_os
pxs.print("===pxs===")
pxs.print('Working Python')
logger = pxs.Logger(";;", False)
pxs.print(logger, "some", "test")
pxs.print("")
pxs.print("===pxs_fs===")
pxs.passert(pxs_fs.File("tests/test_pxs_core.rs").read() == pxs_fs.read_file("tests/test_pxs_core.rs"), "File contents do not match")
pxs_fs.create_dirs("dude/thats/life")
pxs_fs.write_file("dude/thats/life/t.txt", "Dude that is life!")
pxs.passert(pxs_fs.read_file("dude/thats/life/t.txt") == "Dude that is life!", "contents not match")
pxs.passert(len(pxs_fs.read_dir("dude/")) == 1, "length not match")
pxs_fs.remove_dir("dude/")
pxs.print("===pxs_os===")
pxs.print(f"OS name: {pxs_os.name}")
cdir = pxs_os.get_cwd()
pxs_os.chdir("./tests/")
pxs.passert(cdir != pxs_os.get_cwd(), "Directories match! They should not")
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Python error is not null: {:#?}", res);
    }

    fn test_lua() {
        let script = r#"
local pxs = require('pxs')

pxs.print('Working Lua')
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_null(), "Lua error is not null: {:#?}", res);
    }

    fn test_js() {
        let script = r#"
import * as pxs from 'pxs';

pxs.print('Working JS');
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_JavaScript);
        assert!(res.is_null(), "JS error is not null: {:#?}", res);
    }

    #[test]
    fn run_test() {
        println!();
        pxs_initialize();
        pxs_core_initall();

        print_helper("PYTHON");
        test_python();
        // print_helper("LUA");
        // test_lua();
        // print_helper("JS");
        // test_js();

        pxs_finalize();
    }
}
