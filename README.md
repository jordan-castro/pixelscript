[![crates.io](https://img.shields.io/crates/v/pixelscript)](https://crates.io/crates/pixelscript)
[![docs.rs](https://docs.rs/pixelscript/badge.svg)](https://docs.rs/pixelscript)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/jordan-castro/pixelscript?style=social)](https://github.com/jordan-castro/pixelscript)

[![Discord](https://dcbadge.limes.pink/api/server/https://discord.gg/Ws8gp5wSev)](https://discord.gg/Ws8gp5wSev)

# Pixel Script

A multi language scripting runtime built in Rust.

PixelScript lets you expose the same API to multiple different languages in your program. Because it compiles to a C library, you can use it anywhere. 

## Why PixelScript?
Because most games pick only one language for scripting. PixelScript gives modders and scripters a choice:

- Performance? Go with Lua.
- Data/science/prototyping? Choose Python.
- Web developers? You got JavaScript. 

Each language runtime uses the same PixelScript bindings.

## Version
pixelscript crate is currently at version 0.6.6.

## How to use
pixelscript can be used within a rust application or via ffi.

### Rust based
For rust based (i.e. using this library inside a rust application) you can add it with cargo:
```bash
cargo add pixelscript
```

### FFI based
For using pixelscript via ffi, clone this repository and run:
```bash
python scripts/build.py
```
This will build the project and place the necessary *static* libraries in a `/pxsb` folder. It will also generate a `pixelscript.h` C header file.

## Supported languages
| Feature flag     | Language          | Engine                | Notes                           |
|------------------|-------------------|-----------------------|---------------------------------|
| `lua`            | Lua               | [lua](https://lua.org/)                                 | v5.5.                              |
| `python`         | Python            | [pocketpy](https://github.com/pocketpy/pocketpy)        | May require MSVC on Windows        |
| `js`             | JavaScript        | [quickjs-ng](https://github.com/quickjs-ng/quickjs)     | QuickJS-NG small library.          |
<!-- | `easyjs`         | easyjs            | [easyjs](https://github.com/jordan-castro/easyjs)       | Modern syntax, compiles to JS   | -->
<!-- | `php`            | PHP               | PH7                   | Only supports v5.3 and the engine is not maintained anymore | -->

## CoreLib
To include the PixelScript core API, add the `include-core` feature. Or include the specific modules as feature tags.
| Module name | Module purpose |
|-------------|----------------|
| `pxs_json`  | Adds encode/decode functions for all languages.         |
| `pxs_mem`   | Adds memory control to scripting languages.             |
| `pxs_os`    | Adds os functions/helpers.                              |
| `pxs_fs`    | Adds file/directory reading/writing/appending/deleting. |
| `pxs_zip`   | Work with zip files.                                    |
| `pxs_shell` | Direct shell acces.                                     |
| `pxs_http`  | HTTP/S support.                                         |

To read more about the CoreLib, [Read the docs](https://pixelscript.epochtech.us/docs).

<!-- ### pxs_json
Overview of what is incldued in `pxs_json` module.
| Name | Type | Doc Comment |
|------|------|-------------|
| `encode` | Function(pxs_Object) -> pxs_String | Encodes a object into a JSON string. |
| `decode` | Function(pxs_String) -> pxs_Object | Decodes a JSON string into a language object |

### pxs_mem
Overview of what is included in the `pxs_mem` module.
| Name | Type | Doc Comment | Can except |
|------|------|-------------|------------|
| `memdel` | Function(pxs_Objct) | Decreases the refcount for a `PixelObject`. Pass in a `object`, if it does not have `_pxs_ptr` assigned it raises an exception. | No |
| `mem_delall` | Function(pxs_List) | Calls `memdel` sequentially for a `pxs_VarList` of `pxs_Object`s. | No |

### pxs_os
Overview of what is included in `pxs_os` module.
| Name | Type | Doc Comment | Can except |
|------|------|-------------|------------|
| `args`    | pxs_List | Contains the arguments from `std::env::args`. | No |
| `get_cwd` | Function -> pxs_String | Returns the current working directory. | Yes |
| `chdir`   | Function(pxs_String) | Change the current working directory. | Yes |

### pxs_fs
Overview of what is includedin `pxs_fs` module.
| Name | Type | Doc Comment | Can except |
|------|------|-------------|------------|
| `READ_FILE_TEXT`    | pxs_Int64 | To read a file as a pxs_String. | No |
| `READ_FILE_BYTES` | pxs_Int64 | To read a file as a pxs_List[pxs_Byte] | No |
| `read_file`   | Function(pxs_String, pxs_Int64?) | Read a file into a pxs_String or pxs_List[pxs_Byte]. | Yes |
| `write_file`  | Function(pxs_String, pxs_String|pxs_List[pxs_Byte]) | Write into a file. If path does not exist, it will create it. | Yes |
| `exists` | Function(pxs_String) -> pxs_Bool | returns if path exists and is public. | Yes |
| `is_file` | Function(pxs_String) -> pxs_Bool | returns if path is a file. | Yes |
| `is_dir`  | Function(pxs_String) -> pxs_Bool | returns if path is a directory. | Yes |
| `remove_file` | Function(pxs_String) | Removes a file. | Yes |
| `create_dir`  | Function(pxs_String) | Creates a new directory. Is not recursive. | Yes |
| `create_dirs` | Function(pxs_String) | Creates a new directory recursively. | Yes |
| `remove_empty_dir`  | Function(pxs_String) | Removes a empty directory. | Yes |
| `remove_dir` | Function(pxs_String) | Removes a directory regardless if it is not empty. | Yes | -->

## Example
Here is a "Hello World" example supporting Lua, Python, and JavaScript.
```c
#include "pixelscript.h"

// Define a simple `println` function.
pxs_VarT println(pxs_VarT args) {
    pxs_VarT contents_var = pxs_arg(args, 0);
    // We are assuming this is a string.
    char* contents_str = pxs_getstring(contents_var);

    printf("%s", contents_str);

    // Free the string
    pxs_freestr(contents_str);

    // Always required to return something. Null works for no result.
    return pxs_newnull();
}

int main() {
    pxs_initialize();
    
    // Create a module
    pxs_Module* main = pxs_newmod("main");

    // Add callbacks
    pxs_addfunc(main, "println", println);

    // Add module
    pxs_addmod(main);

    // Lua
    const char* lua_script = "local main = require('main')\n"
        "main.println('Hello World from Lua!')";
    pxs_VarT error = pxs_exec(pxs_Lua, lua_script, "<ctest>");
    // Check error
    if (pxs_isexception(error)) {
        char* msg = pxs_getstring(error);
        printf("%s", msg);
        pxs_freestr(msg);
    }
    pxs_freevar(error);

    // Python
    const char* python_script = "import main\n"
                                "main.println('Hello World from Python')\n";

    pxs_VarT error = pxs_exec(pxs_Python, python_script, "<ctest>");
    // Check error
    if (pxs_isexception(error)) {
        char* msg = pxs_getstring(error);
        printf("%s", msg);
        pxs_freestr(msg);
    }
    pxs_freevar(error);

    // JavaScript
    const char* js_script = "import * as main from 'main';\n"
                            "main.println('Hello World from JavaScript!');";
    pxs_VarT error = pxs_exec(pxs_JavaScript, js_script, "<ctest>");
    // Check error
    if (pxs_isexception(error)) {
        char* msg = pxs_getstring(error);
        printf("%s", msg);
        pxs_freestr(msg);
    }
    pxs_freevar(error);

    // All set!

    pxs_finalize();

    return 0;
}
```

## Used in
- Pixel Ai Dash
- [epochweb](https://github.com/jordan-castro/epochweb)

<!-- ## Future -->
<!-- This will ideally be used by all future epochtech games since it allows for modding in multiple languages. 
It's not quite ready to be used in production for anyone other than myself and epochtech. But if you make PRs to fix
something or open issues, I will be responding and merging. Feel free to add a language, just check out /lua or /python for examples on how to use Var, Func, Module, PixelObject, and PixelScripting. -->

Made with ❤️ by [@epochtechgames](https://x.com/epochtechgames)

