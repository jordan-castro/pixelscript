# Gotchas
A list of headaches that I've run into while using this library.

## Python (pocketpy)
- When running `pxs_call` it will go through a checklist until the function is found BY NAME.
    - builtins 
    - current module
    - __main__ module

