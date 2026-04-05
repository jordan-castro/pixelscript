# TODO

- Add a LSP
- Fix warnings (remove or ignore)

## v0.5 Memory and JS support
- JS support via rquickjs. *JS*
- Lazy Init language states on first run.
- ~~Reference counting for PixelScript object.~~ **DONE**
- Decrease number of functions created in pocketpy. *python*
- ~~Return pxs_Exception for errors in pixelscript.~~ **DONE**
- ~~pxs_* library functions that return pxs_VarT need to always return a pxs_Var. nullptr will no longer be allowed.~~ **DONE**
- ~~Reimp pxs_DirHandle to be a pxs_VarList~~ **DONE**
- ~~add pxs_compile which will return a `pxs_Code` object.~~ **DONE**
- ~~Add Map~~ **DONE**
- Review memory management:
    - ~~All functions return pxs_Var~~
    - All functions need to be explicit in their docs on ownership
    - Check Factories... why are we not owning the args?
    - Mark functions as expected return type.
- ~~Add properties to PixelObjects~~ **DONE**
- Add anonyamous functions.
- Use pxs_Map instead for Module variables.

## v0.6 STD and Tests
- Remove lua hacks (io, os, what else?)
- pxs_time (Time functions)
- pxs_os (OS functions like name, version)
- pxs_io (IO functions like write, read, etc) | This will require that `file_loader` `file_reader` and `dir_reader` are setup. 
- Tests
    - test_vars (Test all types to and from scripting)
    - test_all (remove this)
    - test_exec
    - test_eval
    - test_raise (a new test of raising from one langauge to another.)
    - test_ft (a test that builds pixel ai dashs fast terrain system. If this runs, then it most likely works fine.)

## v0.7 Wren support, WASM
- Add Wren support
- WASM support + Wasm web page similar to pocketpy live playground.

## v0

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

## STD
- Add std library via pixelscript runtime. These are optional and handled via features
    - pxs_io (reading/writing files/directories)
    - pxs_os (some OS level stuff (delete, move, idk))
    - pxs_time (similar to pythons time module. Just universal for all languages)

## Lua
- Remove io, os, and hackable modules.

## Python (PocketPy)
- Make callback global. i.e. one per thread
- Make object callbacks global. i.e. one per object
- When dirreader, filereader, filewriter are set, allow with open().
- Add a test for HostObject that holds another HostObject as reference. (FastTerrain)

## JS
- add_variable
- add_callback
- add_module
- execute_javascript
- module_add_variable
- module_add_callback
- module_add_module

## Maybes
- Enums?
- Removing Strings for internal use. I.e. object ids in Python. Try using i32 instead.
- Never return a null pointer? Only use pxs_Var(null)