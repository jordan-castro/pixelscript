// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_vars --no-default-features --features "lua,python,js,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{
        borrow_var, pxs_addfunc, pxs_addmod, pxs_finalize, pxs_freearena, pxs_getbool,
        pxs_getfloat, pxs_getint, pxs_getstring, pxs_getuint, pxs_initialize, pxs_listadd,
        pxs_listget, pxs_map_addpair, pxs_newarena, pxs_newbool, pxs_newexception, pxs_newfloat,
        pxs_newint, pxs_newlist, pxs_newmap, pxs_newmod, pxs_newnull, pxs_newstring, pxs_newuint,
        pxs_varcall, pxs_varis,
        shared::{
            module::pxs_Module,
            pxs_Runtime,
            utils::{self},
            var::{pxs_Var, pxs_VarT, pxs_VarType},
        },
    };
    use etffi::{cstring::CStringSafe, borrow_string, create_raw_string, free_raw_string, own_string, ptr_magic::PtrMagic};

    // We explicitly skip Object, HostObject, Factory, and Exception because those are tested in test_objects.rs

    extern "C" fn test_i64(args: pxs_VarT) -> pxs_VarT {
        let num = pxs_getint(pxs_listget(args, 1));
        println!("num: {num}");

        pxs_newint(-1)
    }

    extern "C" fn test_u64(args: pxs_VarT) -> pxs_VarT {
        let num = pxs_getuint(pxs_listget(args, 1));
        println!("num: {num}");

        pxs_newuint(1)
    }

    extern "C" fn test_string(args: pxs_VarT) -> pxs_VarT {
        let str = own_string!(pxs_getstring(pxs_listget(args, 1)));
        println!("Str: {str}");

        let mut cstgen = CStringSafe::new();
        pxs_newstring(cstgen.new_string("Test"))
    }

    extern "C" fn test_bool(args: pxs_VarT) -> pxs_VarT {
        let bool = pxs_getbool(pxs_listget(args, 1));
        println!("bool: {bool}");

        pxs_newbool(false)
    }

    extern "C" fn test_f64(args: pxs_VarT) -> pxs_VarT {
        let float = pxs_getfloat(pxs_listget(args, 1));
        println!("float: {float}");

        pxs_newfloat(1.2f64)
    }

    extern "C" fn test_null(args: pxs_VarT) -> pxs_VarT {
        let var_is_null = pxs_varis(pxs_listget(args, 1), pxs_VarType::pxs_Null);
        assert!(var_is_null, "Variable is not null");

        pxs_newnull()
    }

    extern "C" fn test_list(args: pxs_VarT) -> pxs_VarT {
        let list = borrow_var!(pxs_listget(args, 1));
        println!("List: {:#?}", list);

        let new_list = pxs_newlist();
        pxs_listadd(new_list, pxs_newint(0));
        pxs_listadd(new_list, pxs_newint(1));
        new_list
    }

    extern "C" fn test_function(args: pxs_VarT) -> pxs_VarT {
        let function = pxs_listget(args, 1);
        // Call it
        let res = pxs_varcall(pxs_listget(args, 0), function, pxs_newlist());
        res
    }

    extern "C" fn test_map(args: pxs_VarT) -> pxs_VarT {
        let map = borrow_var!(pxs_listget(args, 1));
        println!("Map: {:#?}", map);

        let new_map = pxs_newmap();
        let mut cstrgen = CStringSafe::new();
        pxs_map_addpair(
            new_map,
            pxs_newstring(cstrgen.new_string("name")),
            pxs_newstring(cstrgen.new_string("Jordan dayo!")),
        );
        new_map
    }

    // Helper for assertion
    extern "C" fn asrt(args: pxs_VarT) -> pxs_VarT {
        let expr = pxs_getbool(pxs_listget(args, 1));
        if !expr {
            let mut cstrgen = CStringSafe::new();
            pxs_newexception(cstrgen.new_string("Did not pass"));
        }
        pxs_newnull()
    }

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    fn test_python() {
        let script = r#"
from pxs import *
import vars

def num():
    return 1

vars.asrt(vars.test_i64(10) == -1)
vars.asrt(vars.test_u64(2) == 1)
vars.asrt(vars.test_string('Dude') == 'Test')
vars.asrt(vars.test_bool(False) == False)
vars.asrt(vars.test_f64(0.1) == 1.2)
vars.asrt(vars.test_null(None) == None)
vars.asrt(vars.test_list(['Python list']) == [0,1])
vars.asrt(vars.test_function(num) == 1)
vars.asrt(vars.test_map({0: 'Python Map'}) == {'name': 'Jordan dayo!'})
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Python error is not null: {:#?}", res);
    }

    fn test_lua() {
        let script = r#"
local pxs = require('pxs')
local vars = require('vars')

local function num()
    return 1
end

vars.asrt(vars.test_i64(10) == -1)
vars.asrt(vars.test_u64(2) == 1)
vars.asrt(vars.test_string('Dude') == 'Test')
vars.asrt(vars.test_bool(false) == false)
vars.asrt(vars.test_f64(0.1) == 1.2)
vars.asrt(vars.test_null(nil) == nil)
vars.asrt(vars.test_list({'Lua list'})[1] == 0)
vars.asrt(vars.test_function(num) == 1)
vars.asrt(vars.test_map({[0] = 'Lua Map'}).name == 'Jordan dayo!')

"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_null(), "Lua error is not null: {:#?}", res);
    }

    fn test_js() {
        let script = r#"
import * as pxs from 'pxs';
import * as vars from 'vars';

const num = () => {
    return 1;
};

vars.asrt(vars.test_i64(10) == -1)
vars.asrt(vars.test_u64(2) == 1)
vars.asrt(vars.test_string('Dude') == 'Test')
vars.asrt(vars.test_bool(false) == false)
vars.asrt(vars.test_f64(0.1) == 1.2)
vars.asrt(vars.test_null(null) == null)
vars.asrt(vars.test_list(['JS list']) == [0,1])
vars.asrt(vars.test_function(num) == 1)
vars.asrt(vars.test_map({0:'JS Map'}).name == 'Jordan dayo!')
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_JavaScript);
        assert!(res.is_null(), "JS error is not null: {:#?}", res);
    }

    #[test]
    fn run_test() {
        println!();
        pxs_initialize();
        utils::setup_pxs();

        let mut cstrgen = CStringSafe::new();
        let vars_mod = pxs_newmod(cstrgen.new_string("vars"));
        pxs_addfunc(vars_mod, cstrgen.new_string("test_i64"), test_i64);
        pxs_addfunc(vars_mod, cstrgen.new_string("test_u64"), test_u64);
        pxs_addfunc(vars_mod, cstrgen.new_string("test_string"), test_string);
        pxs_addfunc(vars_mod, cstrgen.new_string("test_bool"), test_bool);
        pxs_addfunc(vars_mod, cstrgen.new_string("test_f64"), test_f64);
        pxs_addfunc(vars_mod, cstrgen.new_string("test_null"), test_null);
        pxs_addfunc(vars_mod, cstrgen.new_string("test_list"), test_list);
        pxs_addfunc(vars_mod, cstrgen.new_string("test_function"), test_function);
        pxs_addfunc(vars_mod, cstrgen.new_string("test_map"), test_map);
        pxs_addfunc(vars_mod, cstrgen.new_string("asrt"), asrt);
        pxs_addmod(vars_mod);

        print_helper("PYTHON");
        test_python();
        print_helper("LUA");
        test_lua();
        print_helper("JS");
        test_js();

        pxs_finalize();
    }
}
