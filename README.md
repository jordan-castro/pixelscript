# Pixel Script

A multi language modding runtime built in Rust.

PixelScript lets you expose the same API to multiple different languages in your program. Because it compiles to a C library, you can use it anywhere. 

## Why PixelScript?
Because most games pick only one language for scripting. PixelScript gives modders and scripters a choice:

- Performance? Go with Lua.
- Data/science/prototyping? Choose Python.
- Web developers? You got JavaScript.
- Love easyjs? You got it!

Each language runtime uses the same PixelScript bindings.

## Version
pixelscript crate is currently at version 0.1.0.

## How to use
To use Pixel Script, first install the crate into your rust project.
```bash
cargo add pixelscript
```
And update the `features` flag to include the languages your game/app want.

## Supported languages
| Feature flag     | Language          | Engine                | Notes                       |
|------------------|-------------------|-----------------------|-----------------------------|
| `lua`            | Lua               | mlua                  | Default, fast, battle-tested |
| `lua-jit`        | LuaJit            | mlua                  | does not work on ios        |
| `python`         | Python            | rustpython            | Full stdlib (frozen)        |
| `python-lite`    | Python (light)    | pocketpy              | Smaller binary              |
| `js`             | JavaScript        | boa                   | Pure Rust                   |
| `js-quick`       | JavaScript        | rquickjs              | QuickJS, more complete      |
| `easyjs`         | EasyJS            | Custom                | Requires a JS feature       |
When including `easyjs` make sure to also include a JavaScript feature otherwise it will not work.

## Building
In order to use pixelscript, you need to first compile it's libraries. Each language could potentially have it's own libraries.
Each library will be fetched and placed under a pixel_script folder in the main directory of your build system.

To build simply run:
```bash
cargo build --release
```
On your rust crate. This will build the pixelscript library and make all language libs accessible.

## Example
```c
#include "pixel_script.h"

// ========================== C Binding (START) ==========================

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Define the Struct
typedef struct Person {
    char *name;
    int age;

    // Function pointer to act as a "method"
    void (*print_info)(struct Person *self);
} Person;

// Implementation of the print method
void print_person_info(PixelScriptRuntime runtime, Person *self) {
    if (self != NULL) {
        char* rt = "Unkown";
        if (runtime == Lua) {
            rt = "Lua";
        } else if (runtime == Python) {
            rt = "Python";
        } else if (runtime == JavaScript) {
            rt = "JavaScript";
        } else if (runtime == Easyjs) {
            rt = "EasyJS";
        }
        printf("From runtime: %s, My name is: %s, and I am %d years old\n", rt, self->name, self->age);
    }
}

// "Getter" for Name
const char* get_name(Person *self) {
    return self->name;
}

// "Getter" for Age
int get_age(Person *self) {
    return self->age;
}

// "Setter" for Name
void set_name(Person *self, const char* name) {
    // Free old name
    free(self->name);
    self->name = strdup(name);
} 

// Constructor-like function to initialize the struct
Person* create_person(const char *name, int age) {
    Person *p = malloc(sizeof(Person));
    p->name = strdup(name); // Duplicate string to ensure it persists
    p->age = age;
    return p;
}

// Destructor to free memory
void destroy_person(Person *p) {
    free(p->name);
    free(p);
}

// ========================== C Binding (END) ==========================

Var* ps_set_name(uintptr_t argc, struct Var **argv, void *opaque) {
    Var* object = argv[1];
    // Name is either argv[2] or argv[3] due to the "self" variable that gets passed in via Lua.
    Var* name = argv[3]; // In this example we use LUA, so we will stick to it.

    Person* p = pixelscript_var_get_host_object(object);
    char* new_name = pixelscript_var_get_string(name);

    set_name(p, new_name);

    pixelscript_free_str(new_name);

    return pixelscript_var_newnull();
}

Var* ps_get_name(uintptr_t argc, struct Var **argv, void *opaque) {
    Var* object = argv[1];
    
    Person* p = pixelscript_var_get_host_object(object);

    return pixelscript_var_newstring(p->name);
}

Var* ps_get_age(uintptr_t argc, struct Var **argv, void *opaque) { 
    Var* object = argv[1];

    Person* p = pixelscript_var_get_host_object(object);

    return pixelscript_var_newi64(p->age);
}

Var* ps_greet(uintptr_t argc, struct Var **argv, void *opaque) { 
    Var *runtime = argv[0];
    Var *object = argv[1];

    // Runtime var
    int runtime_int = pixelscript_var_get_i64(runtime);
    Person* p = pixelscript_var_get_host_object(object);

    print_person_info(runtime_int, p);

    return pixelscript_var_newnull();
}

Var* new_person(uintptr_t argc, struct Var **argv, void *opaque) {
    // Assume 1 and 2 are name and age
    Var* name = argv[1];
    Var* age_var = argv[2];

    // Get name string
    char* name_str = pixelscript_var_get_string(name);
    int age = pixelscript_var_get_i64(age_var);

    // Create person
    Person* p = create_person(name_str, age);

    // Free name
    pixelscript_free_str(name_str);

    // Create new object
    PixelObject* object = pixelscript_new_object(p, destroy_person);
    // Add methods
    pixelscript_object_add_callback(object, "set_name", ps_set_name, NULL);
    pixelscript_object_add_callback(object, "get_name", ps_get_name, NULL);
    pixelscript_object_add_callback(object, "get_age", ps_get_age, NULL);
    pixelscript_object_add_callback(object, "greet", ps_greet, NULL);

    // Return object
    return pixelscript_var_newhost_object(object);
}

int main() {
    pixelscript_initialize();

    // Set the new_person object
    pixelscript_add_object("Person", new_person, NULL);
    
    // Lua
    const char* lua_script = "local p = Person('Jordan', 23)\n"
                         "p:greet()\n"
                         "p:set_name('Jordan Castro')\n"
                         "p:greet()\n"
                         "p:set_name('Jordan Castro + ' .. p:get_age())\n"
                         "p:greet()\n";
    char* res = pixelscript_exec_lua(lua_script, "<ctest>");
    pixelscript_free_str(res);

    // Python
    const char* python_script = "p = Person('Jordan', 23)\n"
                                "p.greet()\n"
                                "p.set_name('Jordan Castro')\n"
                                "p.greet()\n"
                                "p.set_name(f'Jordan Castro + {p.get_age()}')\n"
                                "p.greet()\n";

    char* res = pixelscript_exec_python(python_script, "<ctest>");
    pixelscript_free_str(res);

    pixelscript_finalize();

    return 0;
}
```

## Used in
- Pixel Ai Dash
- easyjs (runtime)

## Future
I will not be maintaining this at all. If it works for me that is it, if you have issues or want to make pull requests, feel free and I will look at them.
But this is not production ready at all. If you use this, it is at your own risk.

Made with ❤️ by [@epochtechgames](https://x.com/epochtechgames)