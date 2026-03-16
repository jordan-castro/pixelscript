// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_core --lib --no-default-features --features "lua,python,pxs-debug,include-core" -- --nocapture --test-threads=1

#[cfg(test)]
mod tests {
    use pixelscript::{
        create_raw_string, free_raw_string, own_string, pxs_Opaque, pxs_addfunc, pxs_addmod, pxs_call, pxs_debugvar, pxs_execlua, pxs_execpython, pxs_finalize, pxs_getstring, pxs_initialize, pxs_listadd, pxs_listget, pxs_newcopy, pxs_newint, pxs_newlist, pxs_newmod, pxs_newnull, pxs_tostring, shared::var::pxs_VarT
    };

    extern "C" fn call_pxs_items(args: pxs_VarT, _op: pxs_Opaque) -> pxs_VarT {
        let mname = create_raw_string!("_pxs_items");
        let nargs = pxs_newlist();
        pxs_listadd(nargs, pxs_newcopy(pxs_listget(args, 1)));
        let res = pxs_call(pxs_listget(args, 0), mname, nargs);
        println!("res: {}", own_string!(pxs_debugvar(res)));
        unsafe{
            free_raw_string!(mname);
        }

        return res;
    }

    #[test]
    fn test_globals() {
        pxs_initialize();
        let mname = create_raw_string!("pxs");
        let module = pxs_newmod(mname);
        let fname = create_raw_string!("call_pxs_items");
        pxs_addfunc(module, fname, call_pxs_items, std::ptr::null_mut());
        pxs_addmod(module);
        unsafe {
            free_raw_string!(mname);
            free_raw_string!(fname);
        }

        let pyscript = r#"
from pxs import *

obj = {"one": 1, "two": 2}
items = _pxs_items(obj)

if items != [("one", 1), ("two", 2)]:
    raise "_pxs_items did not work in Python"

# Test calling too
res = call_pxs_items(obj)
print(f"res: {res}")
"#;

        let luascript = r#"
        function deep_compare(t1, t2)
    -- Check if both are tables
    if type(t1) ~= "table" or type(t2) ~= "table" then
        return t1 == t2
    end

    -- Check if they have the same number of elements
    local len1, len2 = 0, 0
    for _ in pairs(t1) do len1 = len1 + 1 end
    for _ in pairs(t2) do len2 = len2 + 1 end
    if len1 ~= len2 then
        return false
    end

    -- For sequential tables, check order and values
    if t1[1] ~= nil and t2[1] ~= nil then
        for i = 1, #t1 do
            if not deep_compare(t1[i], t2[i]) then
                return false
            end
        end
        -- Ensure no extra non-sequential keys
        for k in pairs(t1) do
            if type(k) ~= "number" or k > #t1 or k < 1 then
                return false
            end
        end
        for k in pairs(t2) do
            if type(k) ~= "number" or k > #t2 or k < 1 then
                return false
            end
        end
        return true
    end

    -- For non-sequential tables, compare key-value pairs
    for k, v1 in pairs(t1) do
        local v2 = t2[k]
        if not deep_compare(v1, v2) then
            return false
        end
    end

    return true
end
local obj = {one = 1, two= 2}
items = _pxs_items(obj)
local expected = {{"one", 1}, {"two", 2}}
if not deep_compare(items, expected) then
    error("_pxs_items did not work in lua.")
end"#;

        let raw_pyscript = create_raw_string!(pyscript);
        let raw_file_name = create_raw_string!("<globals_test>");
        let err = own_string!(pxs_execpython(raw_pyscript, raw_file_name));
        unsafe {
            free_raw_string!(raw_pyscript);
        };
        if !err.is_empty() {
            unsafe { free_raw_string!(raw_file_name) };
        }

        assert!(err.is_empty(), "Python error is not empty{err}");

        let raw_luascript = create_raw_string!(luascript);
        let err = own_string!(pxs_execlua(raw_luascript, raw_file_name));
        unsafe {
            free_raw_string!(raw_luascript);
        }
        if !err.is_empty() {
            unsafe { free_raw_string!(raw_file_name) };
        }
        assert!(err.is_empty(), "Lua error is not empty {err}");

        unsafe {
            free_raw_string!(raw_file_name);
        }
        pxs_finalize();
    }
}
