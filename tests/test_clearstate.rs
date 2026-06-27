// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_clearstate --no-default-features --features "lua,python,js,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use std::ffi::CStr;

    use pixelscript::{
        pxs_addvar, pxs_clear, pxs_finalize, pxs_freearena,
        pxs_gethost, pxs_getint, pxs_getuint, pxs_initialize, pxs_listadd, pxs_listget,
        pxs_newarena, pxs_newfactory, pxs_newhost, pxs_newint, pxs_newlist, pxs_newmod,
        pxs_newobject, pxs_newuint,
        shared::{
            module::pxs_Module,
            pxs_Opaque, pxs_Runtime,
            utils::{self},
            var::pxs_VarT,
        },
    };
    use etffi::{cstring::CStringSafe, borrow_string, create_raw_string, free_raw_string, own_string, ptr_magic::PtrMagic};

    #[derive(Clone)]
    struct Vector2 {
        x: i32,
        y: i32,
    }

    impl PtrMagic for Vector2 {}

    extern "C" fn free_v2(ptr: pxs_Opaque) {
        let _ = Vector2::from_raw(ptr as *mut Vector2);
    }

    extern "C" fn new_vector2(args: pxs_VarT) -> pxs_VarT {
        let x = pxs_getint(pxs_listget(args, 1));
        let y = pxs_getint(pxs_listget(args, 2));

        let mut cstrgen = CStringSafe::new();
        let v2 = Vector2 {
            x: x as i32,
            y: y as i32,
        };
        let obj = pxs_newobject(v2.into_void(), free_v2, cstrgen.new_string("Vector2"));
        pxs_newhost(obj)
    }

    /// Generate a Vector2 factory
    fn factory_vector2(v2: Vector2) -> pxs_VarT {
        let args = pxs_newlist();
        pxs_listadd(args, pxs_newint(v2.x as i64));
        pxs_listadd(args, pxs_newint(v2.y as i64));
        pxs_newfactory(new_vector2, args)
    }

    #[derive(Clone)]
    struct Tile {
        atlas: Vector2,
        alt: u32,
        layer: u32,
    }

    impl PtrMagic for Tile {}

    extern "C" fn free_tile(ptr: pxs_Opaque) {
        let _ = Tile::from_raw(ptr as *mut Tile);
    }

    extern "C" fn new_tile(args: pxs_VarT) -> pxs_VarT {
        let mut cstgen = CStringSafe::new();
        let atlas = unsafe {
            Vector2::from_borrow_void(pxs_gethost(pxs_listget(args, 0), pxs_listget(args, 1)))
        };
        let alt = pxs_getuint(pxs_listget(args, 2));
        let layer = pxs_getuint(pxs_listget(args, 3));
        let tile = Tile {
            atlas: Vector2 {
                x: atlas.x,
                y: atlas.y,
            },
            alt: alt as u32,
            layer: layer as u32,
        };
        let obj = pxs_newobject(tile.into_void(), free_tile, cstgen.new_string("Tile"));
        pxs_newhost(obj)
    }

    /// Create a Factory tile
    fn factory_tile(tile: Tile) -> pxs_VarT {
        let factory_args = pxs_newlist();
        pxs_listadd(factory_args, factory_vector2(tile.atlas));
        pxs_listadd(factory_args, pxs_newuint(tile.alt as u64));
        pxs_listadd(factory_args, pxs_newuint(tile.layer as u64));
        pxs_newfactory(new_tile, factory_args)
    }

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    fn test_python() {
        let script = r#"
from pxs import Per, print

for i in range(0,50):
    p = Per('Jordan', 24)

print('Working Python')
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Python);
        assert!(res.is_null(), "Python error is not null: {:#?}", res);
    }

    fn test_lua() {
        let script = r#"
local pxs = require('pxs')

for i = 1, 50 do 
    local p = pxs.Per('Jordan', 24)
end
pxs.print('Working Lua')
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_Lua);
        assert!(res.is_null(), "Lua error is not null: {:#?}", res);
    }

    fn test_js() {
        let script = r#"
import * as pxs from 'pxs';

for (let i = 0; i < 50; i++) {
    let p = pxs.Per('Jordan', 24);
}
pxs.print('Working JS');
"#;
        let res = utils::execute_code(script, "<test>", pxs_Runtime::pxs_JavaScript);
        assert!(res.is_null(), "JS error is not null: {:#?}", res);
    }

    fn setup_module() {
        let mut cstr_safe = CStringSafe::new();

        let module = pxs_newmod(cstr_safe.new_string("module"));

        let tile = Tile {
            alt: 0,
            layer: 0,
            atlas: Vector2 { x: 1, y: 1 },
        };
        for i in 0..50 {
            pxs_addvar(
                module,
                cstr_safe.new_string(&format!("contents{i}")),
                factory_tile(tile.clone()),
            );
        }
    }

    #[test]
    fn run_test() {
        pxs_initialize();
        utils::setup_pxs();

        print_helper("PYTHON");
        for i in 0..50 {
            utils::setup_pxs();
            test_python();
            pxs_clear();
        }
        print_helper("LUA");
        for i in 0..50 {
            utils::setup_pxs();
            test_lua();
            pxs_clear();
        }
        print_helper("JS");
        for i in 0..50 {
            utils::setup_pxs();
            test_js();
            pxs_clear();
        }

        pxs_finalize();
    }
}
