// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_raise --no-default-features --features "lua,python,pxs-debug,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{create_raw_string, free_raw_string, pxs_addmod, pxs_exec, pxs_finalize, pxs_initialize, pxs_newexception, pxs_newmod, shared::{module::pxs_Module, pxs_Runtime, utils, var::pxs_VarT} 
    };
    // pub fn add_function(module: *mut pxs_Module, name: &str, function: pxs_Func) {

    extern "C" fn call(args: pxs_VarT) -> pxs_VarT {
        let msg = create_raw_string!("You no good dayo!!");
        let res = pxs_newexception(msg);
        unsafe {
            free_raw_string!(msg)
        };

        res
    }

    fn setup() {
        let module = utils::create_module("test_raise");
        utils::add_function(module, "call", call);
        pxs_addmod(module);
    }

    fn test_python() {
        let py_script = r#"
import pxs
import test_raise

test_raise.call()

raise Exception("We should not get here")
"#;

    let res = utils::execute_code(py_script, "<test>", pxs_Runtime::pxs_Python);
    assert!(res.is_exception(), "Res is not exeception: {:#?}", res);
    println!("Res: {:#?}", res);
    }

    fn test_lua() {
        let lua_script = r#"
local pxs = require('pxs')
local test_raise = require('test_raise')

test_raise.call()

error('we should not get here')
"#;

    let res = utils::execute_code(lua_script, "<test>", pxs_Runtime::pxs_Lua);
    assert!(res.is_exception(), "Res is not exeception: {:#?}", res);
    println!("Res: {:#?}", res);

    }

    #[test]
    fn run_test() {
        pxs_initialize();
        utils::setup_pxs();
        setup();
        test_python();
        println!("=============== Changing languages =============");
        test_lua();

        pxs_finalize();
    }
}
