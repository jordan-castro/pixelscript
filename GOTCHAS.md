# Gotchas
A list of headaches that I've run into while using this library. Some could be changed, but no promise.

## General
- The first argument in every function call is the runtime it is currently in.
    - This means that function args should be using 1 based index. It's also easy to write a helper function of `get_arg = (args, idx) -> args[idx + 1]`.
- Maps are `pxs_Var` => `pxs_Var` typed.
    - Valid Keys
        - Int64
        - UInt64
        - Float64
        - Bool
        - String
    - Any non valid key will return a `pxs_Exception`.
- Passing `pxs_Object` around runtimes is UB.
    - To implement something similar it's best to implement a custom wrapper system. If the object comes from a `pxs_HostObject` you can recreate the host and 
    pass it into the language. `pxs_Factory` can also be used here for this. So your options are:
    - Custom wrapper system in your host language.
    - Recreate the `pxs_HostObject`.
    - Pass a `pxs_Factory`.

## Python (pocketpy)
- When running `pxs_call` it will go through a checklist until the function is found BY NAME.
    - builtins 
    - current module
    - __main__ module
- When a `pxs_Object` or `pxs_Function` is off the stack and is dropped by pxs. It fails becuase it returns None via `_pxs_register`.
- Does not support inheritance

## JS (quickjs)
- PixelScript does not support returning `undefined`. The only way to do it would be to define a function for JS that returns `undefined`. And use that via 
`pxs_call`. But Python or Lua don't have `undefined`. Python does have `nil` but it would crash the program. So no `undefined` support. For most cases just
use `null`.
- Is currently leaking JSRuntime. I need to review the quickjs-ng source to understand how to free it safely. Something to do with how I am creating the values or something.
  Sucks that it's changes from quickjs where once the runtime is freed it frees all values assigned. But in quickjs-ng this is not the case.
- You have to use `globalThis` to assign (variables, functions) to global scope.