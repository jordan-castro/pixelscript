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
- ~~Review memory management:~~
    - ~~All functions return pxs_Var~~
    - ~~All functions need to be explicit in their docs on ownership~~
    - ~~Check Factories... why are we not owning the args?~~ (We are)
    - ~~Mark functions as expected return type.~~
- ~~Add properties to PixelObjects~~ **DONE**
- Add anonyamous functions. (Get sent to `pxs_anon` module?)
- ~~Check that function calls that fail dont crash.~~
- ~~Add `_pxs_delete` method to free internal memory at language level. (core lib)~~ (it's pxs_mem.memdel(obj))
- ~~Add `arenas`~~
- ~~Promises in JS.~~ (Decided to not support them.)
- Why (globals, locals) are null sometimes?

## v0.6 STD, Tests, Errors
- ~~Use libs/lua-5.5.0/* src instead of mlua.~~
- ~~Remove lua hacks (io, os, what else?)~~
- ~~Add file_name to `pxs_eval`.~~
- ~~Support commonJS.~~
- Tests
    - ~~test_vars (Test all types to and from scripting)~~
    - ~~test_exec~~
    - ~~test_eval~~
    - test_ft (a test that builds pixel ai dashs fast terrain system. If this runs, then it most likely works fine.)
    - make tests smart with features. Pass in specific features in test.py script that overrides the feature in the file.
- Better error messages (as feature 'errors')
    - Explicitly coming from PXS
    - Explicit which runtime
    - Fix JS nasty errors
- Implement `no_std`
    - add `pxs_setalloc`
    - add `pxs_setfree`
    - what else needs to go here?
- Benchmarks
- Add `name` to exceptions. Make it default to `Error` to be backwards compat.
- Remove `pxs_PixelArena` just use `pxs_List`.

## v0.7 Wasm and Dynamic Language support
- Add Wren support
- Add `dynamic` language support meaning a host language can add its own bindings backend that interops perfectly with Pxs.
    - This will be useful when a developer wants to create a custom DSL.
- WASM support + Wasm web page similar to pocketpy live playground.

## v0.8 Cross language
- Cross language support. Calling JS from Python, Lua from JS, Python from JS, etc.

<!-- ## v0.7 Size Reduction -->
<!-- - Remove mlua (use raw lua c files instead) -->
<!-- - Attempting to get pixelscript runtime (not language libraries) <= 10mb -->

## LSP
- Remove ModuleCallbacks just use Function
- Remove ModuleVariables just use PixelVariable {name, var}

## Modules

## Objects
- Drop pxs_Object created from factory when it's no longer needed.

## Vars

## Python (PocketPy)
- Make callback global. i.e. one per thread
- Make object callbacks global. i.e. one per object
- When dirreader, filereader, filewriter are set, allow with open().
- Add a test for HostObject that holds another HostObject as reference. (FastTerrain)

## Maybes
- Enums?
- Removing Strings for internal use. I.e. object ids in Python. Try using i32 instead.
- Never return a null pointer? Only use pxs_Var(null)