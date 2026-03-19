# Gotchas
A list of headaches that I've run into while using this library. That will be changed if possible.

## Python (pocketpy)
- When running `pxs_call` it will go through a checklist until the function is found BY NAME.
    - builtins 
    - current module
    - __main__ module
- When a `pxs_Object` or `pxs_Function` is off the stack and is dropped by pxs. It fails becuase it returns None via `_pxs_register`.
- Does not support inheritance