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
pixelscript crate is currently at version 0.6.3.

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
| `lua`            | Lua               | [lua](https://lua.org/)                                 | v5.5, requires a small shim in `libs/pxs_lua`.       |
| `python`         | Python            | [pocketpy](https://github.com/pocketpy/pocketpy)        | May require MSVC on Windows        |
| `js`             | JavaScript        | [quickjs-ng](https://github.com/quickjs-ng/quickjs)     | QuickJS-NG small library. Supports ES2027 |
<!-- | `easyjs`         | easyjs            | [easyjs](https://github.com/jordan-castro/easyjs)       | Modern syntax, compiles to JS   | -->
<!-- | `php`            | PHP               | PH7                   | Only supports v5.3 and the engine is not maintained anymore | -->

## CoreLib
To include the PixelScript core API, add the `include-core` feature. Or include the specific modules as feature tags.
| Module name | Module purpose |
|-------------|----------------|
| `pxs_json`  | Adds encode/decode functions for all languages. |
| `pxs_mem`   | Adds memory control to scripting languages.     |
<!-- | `pxs_time`  | Adds time functions for all languages. Similar to Python `time` module. | | -->
<!-- | `pxs_io`    | Adds `open`, `File`, `Directory`, `close`, `glob`.      | Requires `pxs_set_filereader`, `pxs_set_filewriter`, and `pxs_set_dirreader` | -->
<!-- | `pxs_os` | -->

### pxs_json
Overview of what is incldued in `pxs_json` module.
| Name | Type | Doc Comment |
|------|------|-------------|
| `encode` | Function | Encodes a object into a JSON string. |
| `decode` | Function | Decodes a JSON string into a language object |

### pxs_mem
Overview of what is included in the `pxs_mem` module.
Call `pxs_meminit` to initialize the module.
| Name | Type | Doc Comment |
|------|------|-------------|
| `memdel` | Function | Decreases the refcount for a `PixelObject`. Pass in a `object`, if it does not have `_pxs_ptr` assigned it raises an exception. |
| `mem_delall` | Function | Calls `memdel` sequentially for a `pxs_VarList` of `pxs_Object`s. |

## Example
Here is a "Hello World" example supporting Lua, Python, and JavaScript.
```c
#include "pixelscript.h"

// Define a simple `println` function.
pxs_VarT println(pxs_VarT args) {
    // Get contents (0 is always Runtime)
    pxs_VarT contents_var = pxs_listget(args, 1);
    // We are assuming this is a string.
    char* contents_str = pxs_getstring(contents_var);

    printf("%s", contents_str);

    // Free the string
    pxs_freestr(contents_str);
}

int main() {
    pxs_initialize();
    
    // Create a module
    pxs_Module* main = pxs_newmod("main");

    // Add callbacks
    pxs_addfunc(main, "println", println, NULL);

    // Add module
    pxs_addmod(main);

    // Lua
    const char* lua_script = "local main = require('main')\n"
        "main.println('Hello World from Lua!')";
    pxs_VarT error = pxs_exec(pxs_Lua, lua_script, "<ctest>");
    // Check error
    if (!pxs_varis(error, pxs_Null)) {
        char* msg = pxs_getstring(error);
        printf("%s", msg);
        pxs_freestr(msg);
    }
    pxs_freevar(error);

    // Python
    const char* python_script = "import main\n"
                                "main.println('Hello World from Python')\n";

    char* error = pxs_exec(pxs_Python, python_script, "<ctest>");
    pxs_freestr(error);

    // JavaScript
    const char* js_script = "import * as main from 'main';\n"
                            "main.println('Hello World from JavaScript!');";
    char* error = pxs_exec(pxs_JavaScript, js_script, "<ctest>");
    pxs_freestr(error);

    pxs_finalize();

    return 0;
}
```

## Used in
- Pixel Ai Dash
- [Yoyo](https://github.com/jordan-castro/yoyo)

<!-- ## Future -->
<!-- This will ideally be used by all future epochtech games since it allows for modding in multiple languages. 
It's not quite ready to be used in production for anyone other than myself and epochtech. But if you make PRs to fix
something or open issues, I will be responding and merging. Feel free to add a language, just check out /lua or /python for examples on how to use Var, Func, Module, PixelObject, and PixelScripting. -->

Made with ❤️ by [@epochtechgames](https://x.com/epochtechgames)

