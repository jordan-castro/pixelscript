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
pixelscript crate is currently at version 0.4.8.

## How to use
To use Pixel Script, you will have to clone this repository.
When compiling, if you only want a specific language you will have to set `--no-default-features` and `--features "<language1>,<language2>"`.
If you want to compile for all languages simply run the build script under `scripts/` It will compile the library and place the files under `/pxsb`.

For rust based use I will be adding a Rust wrapper, which is funny because this is written in Rust. But I want all systems that use the pixelscript library (rust included) to use the low level bindings. Which means I will write a High level and safe rust bindings. Until then, hack it using FFI.

## Supported languages
| Feature flag     | Language          | Engine                | Notes                           |
|------------------|-------------------|-----------------------|---------------------------------|
| `lua`            | Lua               | mlua                  | Fast, battle-tested, v5.4       |
| `python`         | Python            | pocketpy              | Requires MSVC on Windows        |
| `js`             | JavaScript        | rquickjs              | Quickjs, C library              |
| `php`            | PHP               | PH7                   | Only supports v5.3 and the engine is not maintained anymore |
| `rustpython`     | Python (CPython compatible)    | rustpython              | Larger binary, Full Python library support, currently leaking memory.                  |
<!-- | `js-quick`       | JavaScript        | rquickjs              | QuickJS, more complete          | -->

When including `easyjs` make sure to also include a JavaScript feature otherwise it will not work.

## CoreLib
To include the PixelScript core API, add the `include-core` feature. Or include the specific modules as feature tags.
| Module name | Module purpose | Notes |
|-------------|----------------|-------------------|-------|
| `pxs_json`   | Adds JSON encoding, decoding, and .* properties. | Requires a loader function. Set via `pxs_set_file_reader` and a writer function via `pxs_set_file_writer` |
|`pxs_utils` | Adds helpful functions and objects to the std lib. | Check `pxs_utils` for included functions and objects. |

### pxs_utils
Overview of what is incldued in `pxs_utils` module.
| Name | Type | Doc Comment |
|------|------|-------------|
|`_pxs_items`|Function(dict/object/tree) -> returns Array[Array(key, value)]| Converts the key and values of a dictionary (python), object (js), tree (lua) into a list of key,value items.|

<!-- ### Examples
`ps_json` In lua
```lua
local json = require('ps_json')
local data = json.load('path/to/json.json')

-- Now you can read the data
local name = data.name
-- Assuming you have a print wrapper
print(name)
-- Set data
data.name.set("Dude")
-- Set internal
data.fullname.last.set("New")
print(data.fullname)
```
In Python
```python
import ps_json as json
data = json.load('path/to/json.json')

# Read
name = data.name
# Or via dict
name = data['name']
print(name)

data.name.set('Dude')
# Or dict
data['name'] = 'Dude'
# Internal
data.fullname.last.set('New')
print(data.fullname)
```
In JS
```js
import * as ps_json from "ps_json";

let data = ps_json.load('path/to/json.json');
// Read
let name = data.name;
print(name);

data.name.set('Dude');
data['name'] = 'Dude'; // Also works in JS

data.fullname.last.set('New');
print(data.fullname);
```
In easyjs
```easyjs
import 'ps_json'
data = ps_json.load('path/to/json.json');

print(data.name)
print(data['name'])

data.name.set('Dude')
data.fullname.last.set('New')
print(data.fullname)
```
 -->
## Building
In order to use pixelscript, you need to first compile it's libraries. Each language could potentially have it's own libraries.
Each library will be fetched and placed under a pxsb folder in the main directory of your build system.

To build simply run:
```bash
cargo build --release
```
On your rust crate. This will build the pixelscript library and make all language libs accessible.

## Example
Here is a "Hello World" example supporting Lua, Python, PHP, JavaScript, and easyjs.
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
        "main.print('Hello World from Lua!')";
    char* error = pxs_execlua(lua_script, "<ctest>");
    pxs_freestr(error);

    // Python
    const char* python_script = "import main\n"
                                "main.print('Hello World from Python')\n";

    char* error = pxs_execpython(python_script, "<ctest>");
    pxs_freestr(error);

    // JavaScript
    const char* js_script = "import * as main from 'main';\n"
                            "main.print('Hello World from JavaScript!');";
    char* error = pxs_execjs(js_script, "<ctest>");
    pxs_freestr(error);

    // easyjs
    const char* ej_script = "import 'main'\n"
                            "main.print('Hello World from easyjs!')";
    char* error = pxs_execej(ej_script, "<ctest>");
    pxs_freestr(error);

    // PHP!!!! 
    const char* php_script = "include('main');\n" // or require
                            "\\main\\print('Hello World from PHP!');";
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

Also RustPython and Luajit do not currently work.

Made with ❤️ by [@epochtechgames](https://x.com/epochtechgames)