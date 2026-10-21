// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_cross --no-default-features --features "lua,python,js,include-core,testing" -- --nocapture --test-threads=1

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
from pxs import *
from pxs.cross import access


print('========== js ==========')
js = access(2)
js._exec("""
import {print} from 'pxs';

print('from JS!');

// define a function that we are gonna call in a sec.
globalThis.add = (n1, n2) => {
    return n1 + n2;
}

// define a object we are gonna use in a sec
globalThis.Me = {
    name: "Jordan",
    greet(name) {
        print(`Hello ${name}, I am ${this.name}`);
    }
};

""")

js_result = js._call('add', [0,1])
passert(js_result == 1, "Math is not mathing!")

# get Me
js_me = js._get('Me')
print(js_me)
js_me._call('greet', ['Evelyn'])
js_me._set('name', "Jordan Castro")
passert(js_me._get('name') == "Jordan Castro", "Name aint naming!")

print('========== lua =========')
lua = access(0)
lua._exec("""
local pxs = require('pxs')
pxs.print('From Lua!')
""")

print('Working Python')
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
        print_helper("LUA");
        test_lua();
        print_helper("JS");
        test_js();

        pxs_finalize();
    }
}
