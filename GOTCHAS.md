# Gotchas
A list of headaches that I've run into while using this library. That will be changed if possible.

## General
- The first argument in every function call is the runtime it is currently in.
    - This means that function args should be using 1 based index. It's also easy to write a helper function of `get_arg = (args, idx) -> args[idx + 1]`.

## Python (pocketpy)
- When running `pxs_call` it will go through a checklist until the function is found BY NAME.
    - builtins 
    - current module
    - __main__ module
- When a `pxs_Object` or `pxs_Function` is off the stack and is dropped by pxs. It fails becuase it returns None via `_pxs_register`.
- Does not support inheritance

## JS (ejr)
- PixelScript does not support returning `undefined`. The only way to do it would be to define a function for JS that returns `undefined`. And use that via 
`pxs_call`. But Python or Lua don't have `undefined`. Python does have `nil` but it would crash the program. So no `undefined` support. For most cases just
use `null`.