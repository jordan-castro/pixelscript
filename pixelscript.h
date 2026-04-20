// SPDX-License-Identifier: Apache-2.0

#ifndef PIXEL_SCRIPT_H
#define PIXEL_SCRIPT_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * This represents the variable type that is being read or created.
 */
typedef enum pxs_VarType {
  pxs_Int64,
  pxs_UInt64,
  pxs_String,
  pxs_Bool,
  pxs_Float64,
  /**
   * Lua (nil), Python (None), JS/easyjs (null/undefined)
   */
  pxs_Null,
  /**
   * Lua (Tree), Python (Class), JS/easyjs (Prototype)
   */
  pxs_Object,
  /**
   * Host object converted when created.
   * Lua (Tree), Python (object), JS/easyjs (Prototype think '{}')
   */
  pxs_HostObject,
  /**
   * Lua (Tree), Python (list), JS/easyjs (Array)
   */
  pxs_List,
  /**
   * Lua (Value), Python (def or lambda), JS/easyjs (anon function)
   */
  pxs_Function,
  /**
   * Internal object only. It will get converted into the result before hitting the runtime
   */
  pxs_Factory,
  /**
   * Exception is any exception happening at the language level. Pixel Script errors will be caught with pxs_Error in a future release
   */
  pxs_Exception,
  /**
   * A Map Type that ONLY goes from PixelScript to scripting language. You will NEVER receive a Map from a scripting language. It will
   * always default to `pxs_Object`. Does not support all `pxs_VarType`s.
   */
  pxs_Map,
} pxs_VarType;

/**
 * Public enum for supported runtimes.
 */
typedef enum pxs_Runtime {
  /**
   * Lua v5.4 with mlua.
   */
  pxs_Lua = 0,
  /**
   * Python v3.x with pocketpy.
   */
  pxs_Python = 1,
  /**
   * ES 2020 using rquickjs
   */
  pxs_JavaScript = 2,
  pxs_Wren = 3,
} pxs_Runtime;

/**
 * A Factory variable data holder.
 *
 * Holds a callback for creation. And the arguments to be supplied.
 * Runtime will be supplied automatically.
 */
typedef struct pxs_FactoryHolder pxs_FactoryHolder;

/**
 * A Module is a C representation of data that needs to be (imported,required, etc)
 *
 * The process is you add callbacks, variables, etc.
 *
 * And THEN add the module.
 *
 * So you first need to call
 *
 * pixelmods_create_module() Which will create a new module struct with a name.
 *
 * Here is a simple example.
 *
 * ```c
 * Module* m = pixelmods_new_module("math");
 *
 * pixelmods_module_add_callback(m, ...);
 * pixelmods_module_add_variable(m, ...);
 *
 * pixelmods_add_module(m);
 * ```
 *
 * You never free the module pointer because the runtime takes ownership.
 *
 * Callbacks within modules use the same FUNCTION_LOOKUP global static variable.
 */
typedef struct pxs_Module pxs_Module;

/**
 * A PixelScript Object.
 *
 * The way this works is via the host, a Pseudo type can be created. So when the scripting
 * language interacts with the object, it calls it's pseudo methods.
 *
 * example:
 * ```c
 * struct Person {
 *     const char* name;
 *     int age;
 *
 *     Person(const char* name, int age) {
 *         this->name = name;
 *         this->age = age;
 *     }
 *
 *     void set_name(const char* name) {
 *         this->name = name;
 *     }
 *
 *     void set_age(int age) {
 *         this->age = age;
 *     }
 *
 *     int get_age() {
 *         return this->age;
 *     }
 *
 *     const char* get_name() {
 *         return this->name;
 *     }
 * };
 *
 * void free_person(void* p) {
 *     // TODO
 * }
 * Var* person_set_name(int argc, Var** argv, void* opaque) {
 *     Var* object = argv[0];
 *     Person* p = object.value.object_val as Person;
 *     Var* name = argv[1];
 *     p->set_name(name.value.string_val);
 *     return NULL;
 * }
 * Var* new_person(int argc, Var** argv, void* opaque) {
 *     Person* p = malloc();
 *     PixelObject* object_ptr = pixelscript_new_object(p, free_person);
 *     pixelscript_object_add_callback(object_ptr, "set_name", person_set_name);
 *     return pixelscript_var_object(object_ptr);
 * }
 *
 * // OOP base
 * pixelscript_add_object("Person", new_person);
 *
 * // Or functional
 * pixelscript_add_callback("new_person", new_person);
 * ```
 *
 * In a JS example:
 * ```js
 * let p = new Person("Jordan");
 * p.set_name("James");
 * ```
 *
 * This is why a Objects are more like Pseudo types than actual class/objects.
 */
typedef struct pxs_PixelObject pxs_PixelObject;

/**
 * Holds data for a pxs_Var of list.
 *
 * It holds multiple pxsVar within.
 *
 * When creating call:
 *
 * `pixelscript_var_newlist()`.
 *
 * To add items
 *
 * `pixelscript_var_list_add(list_ptr, item_ptr)`
 *
 * To get items
 *
 * `pixelscript_var_list_get(list_ptr, index)`
 *
 * A full example looks like:
 * ```c
 * // Create a new list (you never interact with pxs_VarList directly...)
 * pxs_Var* list = pixelscript_var_newlist();
 *
 * // Add a item
 * pxs_Var* number = pixelscript_var_newint(1);
 * pixelscript_var_list_add(list, number);
 *
 * // Get a item
 * pxs_Var* item_got = pixelscript_var_list_get(list, 0);
 * ```
 */
typedef struct pxs_VarList pxs_VarList;

/**
 * A `Map` in pixelscript is very simply a Key (pxs_Var) to Value (pxs_Var) pair.
 *
 * In Python it's a dictionary, in Lua it's a table, and in JS it's a object.
 */
typedef struct pxs_VarMap pxs_VarMap;

/**
 * A `Object` in pixelscript is wrapped with a potential host_ptr. This allows for non language specific ref counting.
 *
 * To access the raw pointer, use `get_raw()`. Reference counting is automatically applied when this struct is dropped.
 */
typedef struct pxs_VarObject pxs_VarObject;

/**
 * The Variables actual value union.
 */
typedef union pxs_VarValue {
  int64_t i64_val;
  uint64_t u64_val;
  char *string_val;
  bool bool_val;
  double f64_val;
  const void *null_val;
  struct pxs_VarObject *object_val;
  int32_t host_object_val;
  struct pxs_VarList *list_val;
  void *function_val;
  struct pxs_FactoryHolder *factory_val;
  struct pxs_VarMap *map_val;
} pxs_VarValue;

typedef void (*pxs_DeleterFn)(void*);

/**
 * A PixelScript Var(iable).
 *
 * This is the universal truth between all languages PixelScript supports.
 *
 * Currently supports:
 * - int (i32, i64, u32, u64)
 * - float (f32, f64)
 * - string
 * - boolean
 * - Objects
 * - HostObjects (C structs acting as pseudo-classes) This in the Host can also be a Int or Uint.
 * - List
 * - Functions (First class functions)
 *
 * When working with objects you must use the C-api:
 * ```c
 * // Calls a method on a object.
 * pixelscript_object_call(var)
 * ```
 *
 * When using within a callback, if said callback was attached to a Class, the first *mut Var will be the class/object.
 *
 * When using ints or floats, if (i32, u32, u64, f32) there is no gurantee that the supported language uses
 * those types. Usually it defaults to i64 and f64.
 *
 * When creating a object, this is a bit tricky but essentially you have to first create a pointer via the pixel script runtime.
 */
typedef struct pxs_Var {
  /**
   * A tag for the variable type.
   */
  enum pxs_VarType tag;
  /**
   * A value as a union.
   */
  union pxs_VarValue value;
  /**
   * Optional delete method. This is used for Pointers in Objects, and Functions.
   */
  pxs_DeleterFn deleter;
} pxs_Var;

/**
 * Type Helper for a pxs_Var
 * Use this instead of writing out pxs_Var*
 */
typedef struct pxs_Var *pxs_VarT;

/**
 * Function reference used in C.
 *
 * args: *mut pxs_Var, A list of vars.
 * opaque: *mut c_void, opaque user data.
 *
 * Func handles it's own memory, so no need to free the *mut Var returned or the argvs.
 *
 * But if you use any Vars within the function, you will have to free them before the function returns.
 */
typedef struct pxs_Var *(*pxs_Func)(struct pxs_Var *args);

typedef void *pxs_Opaque;

typedef void (*pxs_FreeMethod)(void *ptr);

/**
 * Function Type for Loading a file.
 */
typedef char *(*pxs_LoadFileFn)(const char *file_path);

/**
 * Function Type for writing a file.
 */
typedef void (*pxs_WriteFileFn)(const char *file_path, const char *contents);

/**
 * Function Type for reading a Dir. Should return a `pxs_List`
 */
typedef pxs_VarT (*pxs_ReadDirFn)(const char *dir_path);

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Current pixelscript version.
 */
uint32_t pxs_version(void);

/**
 * Initialize the PixelScript runtime.
 */
void pxs_initialize(void);

/**
 * Finalize the PixelScript runtime.
 */
void pxs_finalize(void);

/**
 * Execute code in a runtime. Will return a pxs_VarT. Null means no error, otherwise error.
 * The result will need to be freed by calling `pxs_freevar`
 */
pxs_VarT pxs_exec(enum pxs_Runtime runtime, const char *code, const char *file_name);

/**
 * Free the string created by the pixelscript library
 *
 * Memory is transfered.
 */
void pxs_freestr(char *string);

/**
 * Create a new pixelscript Module.
 *
 * Can return nullptr.
 */
struct pxs_Module *pxs_newmod(const char *name);

/**
 * Add a callback to a module.
 *
 * Pass in the modules pointer and callback paramaters.
 */
void pxs_addfunc(struct pxs_Module *module_ptr, const char *name, pxs_Func func);

/**
 * Add a Varible to a module.
 *
 * Pass in the module pointer and variable params.
 *
 * Variable ownership is transfered.
 */
void pxs_addvar(struct pxs_Module *module_ptr, const char *name, struct pxs_Var *variable);

/**
 * Add a Module to a Module
 *
 * This transfers ownership.
 */
void pxs_add_submod(struct pxs_Module *parent_ptr, struct pxs_Module *child_ptr);

/**
 * Add the module finally to the runtime.
 *
 * After this you can forget about the ptr since PM handles it.
 */
void pxs_addmod(struct pxs_Module *module_ptr);

/**
 * Optionally free a module if you changed your mind.
 */
void pxs_freemod(struct pxs_Module *module_ptr);

/**
 * Create a new object.
 *
 * This should only be used within a PixelScript function callback. I.e. a constructor.
 *
 * This must be wrapped in a `pxs_newhost` before use within a callback. If setting to a variable, this is done automatically for you.
 *
 * Can return nullptr.
 */
struct pxs_PixelObject *pxs_newobject(pxs_Opaque ptr,
                                      pxs_FreeMethod free_method,
                                      const char *type_name);

/**
 * Add a callback to a object.
 */
void pxs_object_addfunc(struct pxs_PixelObject *object_ptr, const char *name, pxs_Func callback);

/**
 * Add a callback to a object and make it use the language pointer rather than _pxs_ptr idx.
 */
void pxs_object_add_reffunc(struct pxs_PixelObject *object_ptr,
                            const char *name,
                            pxs_Func callback);

/**
 * Add a property to a object. Expects a name and a callback. The same as `pxs_object_addfunc` but that it saves
 * it differently for the backend to convert it into a property.
 */
void pxs_object_addprop(struct pxs_PixelObject *ptr,
                        const char *name,
                        pxs_Func callback);

/**
 * Add a object to a Module.
 *
 * This essentially makes it so that when constructing this Module, this object is instanced.
 * This works by adding a public factory function with the type name. But the type name
 * is mangled (_module_typename).
 *
 * In Lua:
 * ```lua
 * -- Let's say we have a object "Person"
 * local p = Person("Jordan", 23)
 * p:set_name("Jordan Castro")
 * local name = p:get_name()
 *
 * -- Although you could also do
 * local p = Person("Jordan", 23)
 * p.set_name(p, "Jordan") -- You get the idea
 * ```
 *
 * In Python:
 * ```python
 * p = Person("Jordan", 23)
 * # use '.' instead of ':'
 * # etc
 * ```
 *
 * In JS the same as Python and Lua:
 * ```js
 * let p = Person("Jordan", 23);
 * // Same as Python
 * // etc
 * ```
 */
void pxs_addobject(struct pxs_Module *module_ptr, const char *name, pxs_Func object_constructor);

/**
 * Make a new Var string.
 */
pxs_VarT pxs_newstring(const char *str);

/**
 * Make a new Null var.
 */
pxs_VarT pxs_newnull(void);

/**
 * Make a new HostObject var.
 *
 * Transfers ownership
 */
pxs_VarT pxs_newhost(struct pxs_PixelObject *pixel_object);

/**
 * Create a new variable int. (i64)
 */
pxs_VarT pxs_newint(int64_t val);

/**
 * Create a new variable uint. (u64)
 */
pxs_VarT pxs_newuint(uint64_t val);

/**
 * Create a new variable bool.
 */
pxs_VarT pxs_newbool(bool val);

/**
 * Create a new variable float. (f64)
 */
pxs_VarT pxs_newfloat(double val);

/**
 * Call a function on a object, and use a Enum for runtime rather than a var.
 *
 * var is self.
 */
pxs_VarT pxs_object_callrt(enum pxs_Runtime runtime,
                           struct pxs_Var *var,
                           const char *method,
                           struct pxs_Var *args);

/**
 * Object call.
 *
 * All memory is borrowed except for args. But the var returned need to be freed on host side if not returned by a function.
 *
 * You can get the runtime from the first Var in any callback.
 *
 * Example
 * ```C
 *     // Inside a Var* method
 *     Var* obj = pxs_listget(args, 1);
 *     Var name = pxs_object_call()
 * ```
 */
pxs_VarT pxs_objectcall(struct pxs_Var *runtime,
                        struct pxs_Var *var,
                        const char *method,
                        struct pxs_Var *args);

/**
 * Get a int (i64) from a var.
 */
int64_t pxs_getint(struct pxs_Var *var);

/**
 * Get a uint (u64)
 */
uint64_t pxs_getuint(struct pxs_Var *var);

/**
 * Get a float (f64)
 */
double pxs_getfloat(struct pxs_Var *var);

/**
 * Get a Bool
 *
 * CAN_CRASH
 */
bool pxs_getbool(struct pxs_Var *var);

/**
 * Get a String
 *
 * CAN_CRASH, CALLER
 *
 * You have to free this memory by calling `pxs_free_str`
 */
char *pxs_getstring(struct pxs_Var *var);

/**
 * Check if a variable is of a type.
 */
bool pxs_varis(struct pxs_Var *var, enum pxs_VarType var_type);

/**
 * Set a function for reading a file.
 *
 * This is used to load files via import, require, etc
 */
void pxs_set_filereader(pxs_LoadFileFn func);

/**
 * Set a function for writing a file.
 *
 * This is used to write files via pxs_json
 */
void pxs_set_filewriter(pxs_WriteFileFn func);

/**
 * Set a function for reading a directory.
 *
 * This is used to read a dir.
 */
void pxs_set_dirreader(pxs_ReadDirFn func);

/**
 * Free a PixelScript var.
 *
 * You should only free results from `pxs_object_call`
 */
void pxs_freevar(struct pxs_Var *var);

/**
 * Tells PixelScript that we are in a new thread.
 */
void pxs_startthread(void);

/**
 * Tells PixelScript that we just stopped the most recent thread.
 */
void pxs_stopthread(void);

/**
 * Clear the current threads state for all languages.
 *
 * Optionally, if you want to run the garbage collector.
 */
void pxs_clearstate(bool gc_collect);

/**
 * Call a method within a specifed runtime.
 *
 * Runtime is a `pxs_Var`.
 *
 * Transfers ownership of args.
 */
struct pxs_Var *pxs_call(struct pxs_Var *runtime, const char *method, struct pxs_Var *args);

/**
 * Call a ToString method on this Var. If already a string, it won't call it.
 *
 * Host must free this memory with `pxs_free_var`
 */
struct pxs_Var *pxs_tostring(struct pxs_Var *runtime_var, struct pxs_Var *var);

/**
 * Create a new pxs_VarList.
 *
 * This does not take any arguments. To add to a list, you must call `pxs_var_list_add(ptr, item)`
 */
struct pxs_Var *pxs_newlist(void);

/**
 * Add a item to a pxs_VarList.
 *
 * Expects a pointer to pxs_VarList. And a pointer for the item to add (pxs_Var*)
 *
 * This will take ownership of the added item. If you want to copy it instead first create a new `pxs_Var` with `pxs_var_newcopy(item)`
 *
 * Will return the index added at.
 */
int32_t pxs_listadd(struct pxs_Var *list,
                    struct pxs_Var *item);

/**
 * Get a item from a pxs_VarList.
 *
 * Expcts a pointer to pxs_VarList. And a index of i32. Supports negative indexes just like in Python.
 *
 * This will NOT return a cloned variable, you must NOT free it.
 */
struct pxs_Var *pxs_listget(struct pxs_Var *list,
                            int32_t index);

/**
 * Set a item at a specific index in a pxs_VarList.
 *
 * Expects a pointer to pxs_VarList, a index of i32, and a pxs_Var. Supports negative indexes jsut like in Python.
 *
 * Will take ownership of the pxs_Var.
 *
 * This will return a boolean for success = true, or failure = false.
 */
bool pxs_listset(struct pxs_Var *list,
                 int32_t index,
                 struct pxs_Var *item);

/**
 * Get length of a pxs_VarList.
 *
 * Expects a pointer to a pxs_VarList
 */
int32_t pxs_listlen(struct pxs_Var *list);

/**
 * Call a `pxs_Var`s function.
 *
 * Expects runtime var, var function, and args that is a List.
 *
 * Transfers ownership of args.
 */
struct pxs_Var *pxs_varcall(struct pxs_Var *runtime,
                            struct pxs_Var *var_func,
                            struct pxs_Var *args);

/**
 * Copy the pxs_Var.
 *
 * Memory is handled by caller
 */
struct pxs_Var *pxs_newcopy(struct pxs_Var *item);

/**
 * Call a objects getter.
 */
pxs_VarT pxs_objectget(pxs_VarT runtime, pxs_VarT obj, const char *key);

/**
 * Call a objects setter.
 *
 * value ownership is transfered.
 */
bool pxs_objectset(pxs_VarT runtime, pxs_VarT obj, const char *key, pxs_VarT value);

/**
 * Evaluate code. This will return a pxs_Var.
 */
pxs_VarT pxs_eval(const char *script, enum pxs_Runtime rt);

/**
 * Add a factory variable. This variable will be instantiated once at module startup.
 *
 * Args should not contain runtime. It gets added automatically.
 *
 * Basically does:
 * ```python
 * var_name = callback(args)
 * ```
 */
pxs_VarT pxs_newfactory(pxs_Func func, struct pxs_Var *args);

/**
 * Get the HostPointer universally supported for:
 * - Objects that have `_pxs_ptr` assigned.
 * - Integers (signed and unsigned)
 * - HostObjects
 * - Factories (this will call it on the fly.)
 *
 * All other types will return NULL.
 */
void *pxs_gethost(pxs_VarT runtime, pxs_VarT var);

/**
 * Return a string rep of the `pxs_Var`.
 *
 * String must be freed via `pxs_freestr`.
 */
char *pxs_debugvar(pxs_VarT var);

/**
 * Create a `pxs_Exception`.
 */
pxs_VarT pxs_newexception(const char *msg);

/**
 * Get a variable reference from its name
 */
pxs_VarT pxs_var_fromname(pxs_VarT rt, const char *name);

/**
 * Remove a item from a list at a specific index.
 *
 * Returns true for success, false for failed
 */
bool pxs_listdel(pxs_VarT list, int32_t index);

/**
 * Do a Shallow Copy. Which means it gets the same data without get the deleter for (pxs_Object or pxs_Function).
 *
 * Memory is owned by caller.
 */
pxs_VarT pxs_new_shallowcopy(pxs_VarT var);

/**
 * Compile a code string into a code object for later execution.
 *
 * Pass in a optional gloabl scope (or null for default). Scope ownership is transferred.
 * Returns a `pxs_Var` whichs memory is handled by the caller.
 *
 * Resulting `pxs_Var` will contain (Associated Runtime, Code Object, Scope|default).
 */
pxs_VarT pxs_compile(enum pxs_Runtime runtime, const char *code, pxs_VarT global_scope);

/**
 * Execute a compiled code object.
 *
 * Variable ownership is transfered. If this is not desired behavior, pass in a shallow copy.
 * Returned variable must be freed by caller.
 *
 * Runtime is not required because the object is embedded with it in pxs_compile.
 * Pass in a optional local scope that gets passed along with the global scope.
 * Note: Do not use the same scope as in `pxs_compile`.
 *
 * Scope ownership is transferred.
 */
pxs_VarT pxs_execobject(pxs_VarT object, pxs_VarT local);

/**
 * Create a new `pxs_Map`
 */
pxs_VarT pxs_newmap(void);

/**
 * Add a new key (`pxs_Var`) value (`pxs_Var`) pair in a map.
 *
 * Keys can only be:
 * - `pxs_String`
 * - `pxs_Int64`
 * - `pxs_UInt64`
 * - `pxs_Float64`
 * - `pxs_Bool`
 *
 * Key and value ownership are transfered.
 */
void pxs_map_addpair(pxs_VarT map, pxs_VarT key, pxs_VarT value);

/**
 * Remove a value (`pxs_Var`) from a map based on it's key (`pxs_Var`).
 */
void pxs_map_delitem(pxs_VarT map, pxs_VarT key);

/**
 * Get length of a `pxs_Map`.
 *
 * -1 is invalid length.
 */
int32_t pxs_maplen(pxs_VarT map);

/**
 * Get the keys of a `pxs_Map`.
 *
 * Returns a `pxs_List` or `pxs_Null` Which is owned by caller.
 */
pxs_VarT pxs_mapkeys(pxs_VarT map);

/**
 * Get a value in a map from a key.
 *
 * Result is not owned by caller. Use `pxs_newcopy` to transfer ownership.
 */
pxs_VarT pxs_mapget(pxs_VarT map, pxs_VarT key);

/**
 * Insert a item into a list at a certain index, shifting all other items to the right.
 *
 * Item ownership is transferred.
 */
void pxs_listinsert(pxs_VarT list, uintptr_t index, pxs_VarT item);

/**
 * Encode a `pxs_Var` into a JSON string. Will return a `pxs_Var` of type string.
 * Transfers ownership of args.
 * Basically calls the runtime.pxs_json.encode() function.
 *
 * Note: This function is already enabled in each scripting language. This is a host language wrapper for calling it easily.
 */
pxs_VarT pxs_json_encode(pxs_VarT rt,
                         pxs_VarT args);

/**
 * Decode a `pxs_String` into a `pxs_Var`.
 * Make sure runtime is the first argument in args.
 * Transfers ownership of args.
 *
 * Note: This function is already enabled in each scripting language. This is a host language wrapper for calling it easily.
 */
pxs_VarT pxs_json_decode(pxs_VarT rt,
                         pxs_VarT args);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* PIXEL_SCRIPT_H */
