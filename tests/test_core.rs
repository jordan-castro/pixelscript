// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_core --lib --no-default-features --features "lua,python,pxs-debug,include-core" -- --nocapture --test-threads=1
#[allow(unused)]

#[cfg(test)]
mod tests {
    use pixelscript::{
        create_raw_string, free_raw_string, own_string, own_var, pxs_Opaque, pxs_addfunc, pxs_addmod, pxs_call, pxs_debugvar, pxs_exec, pxs_finalize, pxs_getstring, pxs_initialize, pxs_json_decode, pxs_json_encode, pxs_listadd, pxs_listget, pxs_new_shallowcopy, pxs_newcopy, pxs_newint, pxs_newlist, pxs_newmod, pxs_newnull, pxs_tostring, shared::{PtrMagic, pxs_Runtime, var::{pxs_Var, pxs_VarT}}
    };

    extern "C" fn call_pxs_json_encode(args: pxs_VarT) -> pxs_VarT {
        let rt = pxs_listget(args, 0);
        let obj = pxs_listget(args, 1);

        let nargs = pxs_newlist();
        pxs_listadd(nargs, pxs_new_shallowcopy(obj));
        let res = pxs_json_encode(rt, nargs);
        res
    } 

    extern "C" fn call_pxs_json_decode(args: pxs_VarT) -> pxs_VarT {
        let rt = pxs_listget(args, 0);
        let obj = pxs_listget(args, 1);

        let nargs = pxs_newlist();
        pxs_listadd(nargs, pxs_new_shallowcopy(obj));
        let res = pxs_json_decode(rt, nargs);
        res
    }

    #[test]
    fn test_globals() {
        pxs_initialize();
        let mname = create_raw_string!("pxs");
        let module = pxs_newmod(mname);
        let fname = create_raw_string!("encode");
        let fname2 = create_raw_string!("decode");
        pxs_addfunc(module, fname, call_pxs_json_encode);
        pxs_addfunc(module, fname2, call_pxs_json_decode);
        pxs_addmod(module);
        unsafe {
            free_raw_string!(mname);
            free_raw_string!(fname);
            free_raw_string!(fname2);
        }

        let pyscript = r#"
from pxs import *
obj = {"one": 1, "two": 2}
encoded = pxs_json.encode(obj)
print(f'encoded: {encoded}')
decoded = pxs_json.decode(encoded)
print(f'decoded == obj: {decoded == obj}')

encoded2 = encode(obj)
print(f'encoded2: {encoded2}')
decoded2 = decode(encoded2)
print(f'decoded2 == obj: {decoded2 == obj}')
"#;

        let luascript = r#"
local pxs = require('pxs')
local obj = {one = 1, two= 2}
local encoded = pxs_json.encode(obj)
print('encoded: ' .. encoded)
local decoded = pxs_json.decode(encoded)
print(decoded.one)
print(decoded.two)

local encoded2 = pxs.encode(obj)
local decoded2 = pxs.decode(encoded2)

print('encoded2: ' .. encoded2)
print(decoded2.one)
print(decoded2.two)
"#;

        let raw_pyscript = create_raw_string!(pyscript);
        let raw_file_name = create_raw_string!("<globals_test>");
        let err = own_var!(pxs_exec(pxs_Runtime::pxs_Python, raw_pyscript, raw_file_name));
        unsafe {
            free_raw_string!(raw_pyscript);
        };
        if !err.is_null() {
            unsafe { free_raw_string!(raw_file_name) };
        }

        assert!(err.is_null(), "Python error is not empty{}", err.get_string().unwrap());

        let raw_luascript = create_raw_string!(luascript);
        let err = own_var!(pxs_exec(pxs_Runtime::pxs_Lua, raw_luascript, raw_file_name));
        unsafe {
            free_raw_string!(raw_luascript);
        }
        if !err.is_null() {
            unsafe { free_raw_string!(raw_file_name) };
        }
        assert!(err.is_null(), "Lua error is not empty {}", err.get_string().unwrap());

        unsafe {
            free_raw_string!(raw_file_name);
        }
        pxs_finalize();
    }
}
