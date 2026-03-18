[<image-card alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue" ></image-card>](LICENSE)
[<image-card alt="Stars" src="https://img.shields.io/github/stars/jordan-castro/pixelscript" ></image-card>](https://github.com/jordan-castro/pixelscript)

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
pixelscript crate is currently at version 0.5.1.

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
| `lua`            | Lua               | mlua                  | Fast, battle tested, v5.4       |
| `python`         | Python            | pocketpy V2.1.8       | Requires MSVC on Windows        |
| `js`             | JavaScript        | rquickjs              | Quickjs rust wrapper            |
| `php`            | PHP               | PH7                   | Only supports v5.3 and the engine is not maintained anymore |
<!-- | `rustpython`     | Python            | rustpython            | Larger binary, Full Python library support, currently leaking memory. | -->
<!-- | `luajit`         | Lua               | mlua                  | Uses the same code as the `lua` feature | -->

## CoreLib
To include the PixelScript core API, add the `include-core` feature. Or include the specific modules as feature tags.
| Module name | Module purpose | Notes |
|-------------|----------------|-------|
| `pxs_json`  | Adds encode/decode functions for all languages. | |

### pxs_json
Overview of what is incldued in `pxs_json` module.
| Name | Type | Doc Comment |
|------|------|-------------|
| `encode` | Function | Encodes a object into a JSON string. |
| `decode` | Function | Decodes a JSON string into a language object |

## Example
Here is a "Hello World" example supporting Lua, Python, JavaScript and PHP.
```c
#include "pixelscript.h"
// Optional macros (for C/C++ codespaces)
#include "pixelscript_m.h"

// One with the macro
PXS_HANDLER(mprintln) {
    pxs_VarT contents_var = PXS_ARG(1);
    char* contents_str = pxs_getstring(contents_var);

    printf("%s", contents_str);

    // Free the string
    pxs_freestr(contents_str);
}

// One without the macro
pxs_VarT println(pxs_VarT args, pxs_Opaque opaque) {
    // Get contents
    pxs_VarT contents_var = pxs_listget(args, 1);
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
    pxs_addfunc(main, "mprintln", mprintln, NULL);
    pxs_addfunc(main, "println", println, NULL);

    // Lua
    const char* lua_script = "local main = require('main')\n"
        "main.println('Hello World from Lua!')";
    char* error = pxs_execlua(lua_script, "<ctest>");
    pxs_freestr(error);

    // Python
    const char* python_script = "import main\n"
                                "main.println('Hello World from Python')\n";

    char* error = pxs_execpython(python_script, "<ctest>");
    pxs_freestr(error);

    // JavaScript
    const char* js_script = "import * as main from 'main';\n"
                            "main.println('Hello World from JavaScript!');";
    char* error = pxs_execjs(js_script, "<ctest>");
    pxs_freestr(error);

    // PHP!!!! 
    const char* php_script = "include('main');\n" // or require
                            "\\main\\println('Hello World from PHP!');";
    char* error = pxs_execphp(php_script, "<ctest>");
    pxs_freestr(error);

    pxs_finalize();

    return 0;
}
```

## Used in
- Pixel Ai Dash
- easyjs (runtime)

## Future
This will ideally be used by all future epochtech games since it allows for modding in multiple languages. 
It's not quite ready to be used in production for anyone other than myself and epochtech. But if you make PRs to fix
something or open issues, I will be responding and merging. Feel free to add a language, just check out /lua or /python for examples on how to use Var, Func, Module, PixelObject, and PixelScripting.

<!-- Also RustPython and Luajit do not currently work. -->

Made with ❤️ by [@epochtechgames](https://x.com/epochtechgames)