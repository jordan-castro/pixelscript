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
    use etffi::{
        borrow_string, create_raw_string, cstring::CStringSafe, free_raw_string, own_string,
        ptr_magic::PtrMagic,
    };
    use pixelscript::{
        pxs_core_initall, pxs_finalize, pxs_freearena, pxs_initialize, pxs_newarena, pxs_newmod,
        shared::{module::pxs_Module, pxs_Runtime, utils, var::pxs_VarT},
    };

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

globalThis.add = (n1, n2) => {
    return n1 + n2;
}

globalThis.Me = {
    name: "Jordan",
    greet(name) {
        print(`Hello ${name}, I am ${this.name}`);
    }
};
""")

js_result = js._call('add', [0, 1])
passert(js_result == 1, "Math is not mathing!")

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

function add(n1, n2)
    return n1 + n2
end

Me = {
    name = "Jordan"
}
""")

lua_result = lua._call('add', [0, 1])
passert(lua_result == 1, "LUA Math is not mathing!")

lua_me = lua._get('Me')
passert(lua_me._get('name') == 'Jordan', 'Lua name bad')
lua_me._set('name', 'Jordan Castro')
passert(lua_me._get('name') == 'Jordan Castro', 'lUA new name bad')

print('Working Python')
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Python error is not null: {:#?}", res);
    }

    fn test_lua() {
        let script = r#"
local pxs = require('pxs')
local passert = pxs.passert
local access = require('pxs.cross').access

local py = access(1)
local js = access(2)

pxs.print("====== Python via Lua ======")
py:_exec([[
def add(n1, n2):
    return n1 + n2

class MeObj:
    def __init__(self):
        self.name = "Jordan"

Me = MeObj()
]])

local py_result = py:_call('add', {0, 1})
passert(py_result == 1, "Python math from Lua failed!")

local py_me = py:_get('Me')
passert(py_me:_get('name') == 'Jordan', 'Python name from Lua failed')
py_me:_set('name', 'Jordan Castro')
passert(py_me:_get('name') == 'Jordan Castro', 'Python new name from Lua failed')

pxs.print("====== JS via Lua ======")
js:_exec([[
globalThis.add = (n1, n2) => n1 + n2;
globalThis.Me = { name: "Jordan" };
]])

local js_result = js:_call('add', {0, 1})
passert(js_result == 1, "JS math from Lua failed!")

local js_me = js:_get('Me')
passert(js_me:_get('name') == 'Jordan', 'JS name from Lua failed')
js_me:_set('name', 'Jordan Castro')
passert(js_me:_get('name') == 'Jordan Castro', 'JS new name from Lua failed')

pxs.print('Working Lua')
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_null(), "Lua error is not null: {:#?}", res);
    }

    fn test_js() {
        let script = r#"
import * as pxs from 'pxs';
import { access } from 'pxs.cross';

const passert = pxs.passert;

const lua = access(0);
const py = access(1);

pxs.print('========== Lua via JS ==========');
lua._exec(`
local pxs = require('pxs')
function add(n1, n2)
    return n1 + n2
end
Me = { name = "Jordan" }
`);

const lua_result = lua._call('add', [0, 1]);
passert(lua_result === 1, 'Lua math from JS failed!');

const lua_me = lua._get('Me');
passert(lua_me._get('name') === 'Jordan', 'Lua name from JS failed');
lua_me._set('name', 'Jordan Castro');
passert(lua_me._get('name') === 'Jordan Castro', 'Lua new name from JS failed');

pxs.print('========== Python via JS ==========');
py._exec(`
def add(n1, n2):
    return n1 + n2

class MeObj:
    def __init__(self):
        self.name = "Jordan"

Me = MeObj()
`);

const py_result = py._call('add', [0, 1]);
passert(py_result === 1, 'Python math from JS failed!');

const py_me = py._get('Me');
passert(py_me._get('name') === 'Jordan', 'Python name from JS failed');
py_me._set('name', 'Jordan Castro');
passert(py_me._get('name') === 'Jordan Castro', 'Python new name from JS failed');

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
