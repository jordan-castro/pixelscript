# TODO

- Make better use of anyhow
- Make better use of '?'
- C tests
    - Lua
    - Python
    - JS
    - Easyjs

- Add a LSP
- Fix warnings (remove or ignore)

## v0.5
- Lazy Init
- Deprecate opaque objects. They are not needed because of our HostObject system.
- Reference counting for PixelScript object
- pxs_Error (returned when there is a error in pixelscript not necessarily a error in a backend)
- Add pxs_Ref variable which will take a string name and at conversion grab it from the scope.
    - In Lua you can grab it de una vez
    - In pocketpy you do current module ? globals ? result

## v0.6
- pxs DSL

## LSP
- Remove ModuleCallbacks just use Function
- Remove ModuleVariables just use PixelVariable {name, var}

## Functions
- Use VarList instead of argc and argv?

## Modules

## Objects
- Drop pxs_Object created from factory when it's no longer needed.

## Vars
- ~~Add exceptions~~
- Add Map
- Copy might need to be a little smarter because right now if you copy a object and pass it into a function that takes ownership or the 
    arg it will drop it. So either we allow a copy without deleter function. Or we internally recreate a new reference,
    I think pxs_copy_nodelete() is a good idea. But I would rather not add a new lib function. Rather if something could be done internally that is 
    not too complex like `PythonPointer`. That would be better.

## STD
- Add std library via pixelscript runtime. These are optional and handled via features
    - io (reading/writing files/directories)
    - os (some OS level stuff (delete, move, idk))

## Lua
- Remove io, os, and hackable modules.

## Python (rustpython)
- Eventually look back at this

## Python (PocketPy)
- Make callback global. i.e. one per thread
- Make object callbacks global. i.e. one per object
- When dirreader, filereader, filewriter are set, allow with open().
- ~~Remove the pointer math for the pointer types and shit.~~

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
