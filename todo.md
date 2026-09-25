# TODO

## v0.5 Memory and JS support
- ~~JS support via quickjs-ng. *JS*~~ **DONE**
- ~~Reference counting for PixelScript object.~~ **DONE**
- ~~Decrease number of functions created in pocketpy.~~ *python*
- ~~Return pxs_Exception for errors in pixelscript.~~ **DONE**
- ~~pxs_* library functions that return pxs_VarT need to always return a pxs_Var. nullptr will no longer be allowed.~~ **DONE**
- ~~Reimp pxs_DirHandle to be a pxs_VarList~~ **DONE**
- ~~add pxs_compile which will return a `pxs_Code` object.~~ **DONE**
- ~~Add Map~~ **DONE**
- ~~Review memory management:~~ **DONE**
    - ~~All functions return pxs_Var~~
    - ~~All functions need to be explicit in their docs on ownership~~
    - ~~Check Factories... why are we not owning the args?~~ (We are)
    - ~~Mark functions as expected return type.~~
- ~~Add properties to PixelObjects~~ **DONE**
- ~~Check that function calls that fail dont crash.~~ **DONE**
- ~~Add `_pxs_delete` method to free internal memory at language level. (core lib)~~ (it's pxs_mem.memdel(obj)) **DONE**
- ~~Add `arenas`~~ **DONE**
- ~~Promises in JS.~~ (Decided to not support them.)
- ~~Why (globals, locals) are null sometimes?~~ (because sometimes they are literally not passed.)

## v0.6 STD, Tests, Errors
- ~~Use libs/lua-5.5.0/* src instead of mlua.~~ **DONE**
- ~~Remove lua hacks (io, os, what else?)~~ **DONE**
- ~~Add file_name to `pxs_eval`.~~ **DONE**
- ~~Support commonJS.~~ **DONE**
- ~~Tests~~ **DONE**
    - ~~test_vars (Test all types to and from scripting)~~
    - ~~test_exec~~
    - ~~test_eval~~
    - ~~test_ft (a test that builds pixel ai dashs fast terrain system. If this runs, then it most likely works fine.)~~
- Better error messages (as feature 'errors')
    - Explicitly coming from PXS
    - Explicit which runtime
    - Fix JS nasty errors
- Implement `no_std`
    - add `pxs_setalloc`
    - add `pxs_setfree`
    - what else needs to go here?
- ~~Benchmarks~~ (This will NOT be added. Mostly becuase as I started writing them I realized that what needs to be benchmarked are the BACKENDS. Which
already have their own benchmarks in the own repos. Please look at their documentation to see benchmarks for each backend. It is backend driven afterall.)
- Add `name` to exceptions. Make it default to `Error` to be backwards compat.
- ~~Update how when adding child modules to a module it changes the names correctly. (Only do this at final pxs_addmod).~~ **DONE**
- Add more `pxs` core modules
    - ~~add `pxs_os`   ~~
    - ~~add `pxs_fs`   ~~
    - ~~add `pxs_shell`~~
    - add `pxs_zip`
    - add `pxs_parser`
        - `json`
        - `yaml`
        - `toml`
        - `csv`
        - the parser library is going to be different than `pxs_json` because `pxs_json` is mainly just a way of adding JSON support to the host via the library.
            but that will be deprecated in favor of the new parser library.
    - ~~add `pxs_http`~~ (Converted to Rust.)
    - ~~Important caveat with core modules: THEY MUST NOT USE ANY CRATES! So zip and http are written in C++.~~ (This is wrong now, I actually need to convert
    those modules into rust but without adding crates. So they will be using extern c callbacks. For zip I will either write a library or remove it from core.).

- Add android build support in `build.py`
- ~~Add `zigbuild` support in `build.py`~~ **DONE**
- ~~Fix child modules naming.~~ **DONE**
    - ~~Should be renamed when adding module to another module type thingy.~~
- ~~Remove `c_tests`~~ **DONE**
- ~~Add name to `compile`.~~ **DONE**
- ~~remove locals from python and JS backends. Make them override the global scope.~~ **DONE**
    - ~~in Python (just set and unset keys) (never pass locals. Or do but pass a nulll?)~~
    - ~~in JS (do the same thing...)~~
- ~~update pocketpy version.~~ **DONE**
- ~~Fix the python memory leak of locals not being removed from globals.~~ (Not doing this anymore.)
- Add stack errors. This is just a regular pxs_Exception that is programmed to also save a stack msg.
- Update python backend to not write raw strings for object creation.
- Add custom bindings:
    - ~~pocketpy~~
    - lua
    - quickjsng
    This is to support WASM, better cross platform control, and remove (bindgen, cbindgen) from build dependencies.


## v0.7 Wasm and Docs ~~and Dynamic Language support~~ (Dynamic support will not be supported. If you want to add a custom language)
- ~~Add `dynamic` language support meaning a host language can add its own bindings backend that interops perfectly with Pxs.~~ (Developers should add fork and add their own backend following the docs. If they want to add it to pixelscript they will need to do a PR.)
    ~~- This will be useful when a developer wants to create a custom DSL.~~
- WASM support + Wasm web page similar to pocketpy live playground. (at pixelscript.epochtech.us/playground)
- Write documentation at (pixelscript.epochtech.us)

## v0.8 Cross language, Binary (pxs), Wren backend
- Cross language support. Calling JS from Python, Lua from JS, Python from JS, etc.
- Add a pxs binary for:
    - Compiling PXS programs.
    - Using pixelscript as a full runtime.
- Add Wren support

## v0.9 API, Backends
- Add `pxs_addmod2(module:pxs_Module, runtimes:pxs_List[pxs_Int])` which would take a module and a runtime so that you add a specific module to specific runtimes.
- Add C python api support.
- Add Node JS api support.
- Add C python backend.
- Add Add V8 backend.

## Maybes
- Enums?
- Removing Strings for internal use. I.e. object ids in Python. Try using i32 instead.
- Never return a null pointer? Only use pxs_Var(null)