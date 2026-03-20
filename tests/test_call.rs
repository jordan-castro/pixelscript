// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_call --no-default-features --features "lua,python,pxs-debug,include-core" -- --nocapture --test-threads=1

#[allow(unused)]
#[cfg(test)]
mod tests {
    use pixelscript::{
        create_raw_string, free_raw_string, own_string, own_var, pxs_Opaque, pxs_addfunc, pxs_addmod, pxs_call, pxs_debugvar, pxs_exec, pxs_finalize, pxs_freevar, pxs_getint, pxs_initialize, pxs_listadd, pxs_listget, pxs_newint, pxs_newlist, pxs_newmod, pxs_newnull, shared::{pxs_Runtime, var::pxs_VarT, var::pxs_Var, PtrMagic}
    };

    extern "C" fn anything(args: pxs_VarT) -> pxs_VarT {
        let mn = create_raw_string!("add");
        let iargs = pxs_newlist();
        pxs_listadd(iargs, pxs_newint(1));
        pxs_listadd(iargs, pxs_newint(2));
        let res = pxs_call(pxs_listget(args, 0), mn, iargs);
        println!("{}", own_string!(pxs_debugvar(res)));
        assert!(pxs_getint(res) == 3, "We could not run the add function");
        pxs_freevar(res);
        unsafe {free_raw_string!(mn); }
        return pxs_newnull();
    }

    #[test]
    fn test_call() {
        pxs_initialize();

        let mod_name = create_raw_string!("pxs");
        let module = pxs_newmod(mod_name);
        let anything_name = create_raw_string!("anything");
        pxs_addfunc(module, anything_name, anything);
        pxs_addmod(module);

        let script = create_raw_string!(r#"
from pxs import *
def add(n1, n2):
    return n1 + n2

anything(1,2)
"#);
        let file_name = create_raw_string!("<test>");

        let err = own_var!(pxs_exec(pxs_Runtime::pxs_Python, script, file_name));

        unsafe{
free_raw_string!(script);
free_raw_string!(file_name);
free_raw_string!(mod_name);
free_raw_string!(anything_name);
        };

        assert!(err.is_null(), "Error is not empty: {}", err.get_string().unwrap());

        pxs_finalize();
    }
}
