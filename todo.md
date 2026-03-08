# TODO

- add rust first functions? (i.e. in lib add rust specific functions.)

- file io
    - add_loader_callback (file_loader)
    - Optional

- Make better use of anyhow
- Make better use of '?'
- C tests
    - Lua
    - Python
    - JS
    - Easyjs

- Add a LSP

## v0.4
- Lazy Init
- Deprecate opaque objects. They are not needed because of our HostObject system.

## LSP
- Remove ModuleCallbacks just use Function
- Remove ModuleVariables just use PixelVariable {name, var}

## Functions
- Use VarList instead of argc and argv?

## Modules

## Objects
- Drop pxs_Object created from factory when it's no longer needed.

## Vars
- Add exceptions
- Add Map

## STD
- Add std library via pixelscript runtime. These are optional and handled via features
    - json
    - math
    - io
    - os

## Lua
- Remove io, os, and hackable modules.
- Add _pxs_items global

## Python (rustpython)
- Eventually look back at this

## Python (PocketPy)
- Make callback global. i.e. one per thread
- Make object callbacks global. i.e. one per object
- Add tuple as pxs_List
- Add _pxs_items global

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
- Instead of defining everything AOT. Define it JIT for memory.