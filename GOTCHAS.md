# Gotchas
A list of headaches that I run into while using this library.

## Python (pocketpy)
- When running `pxs_call` it will go through a checklist until the function is found BY NAME.
    - builtins 
    - current module
    - __main__ module
- `pxs_Function` can only be called within the function that accepts it as a arg.