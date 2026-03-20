# TODO

- Add a LSP
- Fix warnings (remove or ignore)

## v0.5 Memory and JS support
- JS support via ejr.
- Lazy Init language states on first run.
- Reference counting for PixelScript object.
- Decrease number of functions created in pocketpy.
- pxs_Error (returned when there is a error in pixelscript not necessarily a error in a backend) This will replace anyhow.
- pxs_* library functions need to always return a pxs_Var. nullptr will no longer be allowed.
- ~~Reimp pxs_DirHandle to be a pxs_VarList~~

## v0.6 Platforms and STD
- pxs DSL (Might not do this TBH.)
- WASM support + Wasm web page similar to pocketpy live playground.
- pxs_time (Time functions)
- pxs_os (OS functions like name, version)
- pxs_io (IO functions like write, read, etc) | This will require that `file_loader` `file_reader` and `dir_reader` are setup. 

## v0.7 Size Reduction
- Remove mlua (use raw lua c files instead)
- Attempting to get pixelscript <= 10mb

## LSP
- Remove ModuleCallbacks just use Function
- Remove ModuleVariables just use PixelVariable {name, var}

## Modules

## Objects
- Drop pxs_Object created from factory when it's no longer needed.

## Vars
- Add Map

## STD
- Add std library via pixelscript runtime. These are optional and handled via features
    - pxs_io (reading/writing files/directories)
    - pxs_os (some OS level stuff (delete, move, idk))
    - pxs_time (similar to pythons time module. Just universal for all languages)

## Lua
- Remove io, os, and hackable modules.
- Drop down to raw C lua lib...

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