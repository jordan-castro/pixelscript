# TODO

- add rust first functions? (i.e. in lib add rust specific functions.)

- file io
    - add_loader_callback (file_loader)
    - Optional

- Make better use of anyhow
- C tests
    - Lua
    - Python
    - JS
    - Easyjs

- Add a LSP

## LSP
- Remove ModuleCallbacks just use Function
- Remove ModuleVariables just use PixelVariable {name, var}

## Functions
<!-- - Use a Vector for lookup. -->

## Modules

## Objects
<!-- - Use a Vector for lookup. -->
- Optomize Object creation. Potentially doing at `add` rather than callbacks.

## Vars
- add Array type which holds many Vars.
- Add to_* methods for bool, int, float, and string. These methods will use the Object.call and will require the runtime.

## STD
- Add std library via pixelscript runtime. These are optional and handled via features
    - json
    - math
    - io
    - os

## Lua

## Python (rustpython)
- Eventually look back at this

## Python (PocketPy)
- custom imports

## JS
- add_variable
- add_callback
- add_module
- execute_javascript
- module_add_variable
- module_add_callback
- module_add_module

## easyjs
- add_variable
- add_callback
- add_module
- execute_easyjs
- module_add_variable
- module_add_callback
- module_add_module

## Starlark ([url](https://github.com/facebook/starlark-rust))
- add_variable
- add_callback
- add_module
- add_object
- module_add_variable
- module_add_callback
- module_add_object
- object.call
- custom imports
- execute_starlark
