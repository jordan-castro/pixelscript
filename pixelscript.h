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
} pxs_VarType;

/**
 * Public enum for supported runtimes.
 */
typedef enum pxs_Runtime {
  /**
   * Lua v5.4 with mlua.
   */
  pxs_Lua,
  /**
   * Python v3.x with pocketpy.
   */
  pxs_Python,
  /**
   * ES 2020 using rquickjs
   */
  pxs_JavaScript,
  /**
   * v0.4.5 using easyjsc
   */
  pxs_Easyjs,
  /**
   * Python >= v3.8 with RustPython
   */
  pxs_RustPython,
  /**
   * PHP v5.3 with PH7
   */
  pxs_PHP,
} pxs_Runtime;

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
 * The Variables actual value union.
 */
typedef union pxs_VarValue {
  int64_t i64_val;
  uint64_t u64_val;
  char *string_val;
  bool bool_val;
  double f64_val;
  const void *null_val;
  void *object_val;
  int32_t host_object_val;
  struct pxs_VarList *list_val;
  void *function_val;
} pxs_VarValue;

typedef void (*DeleterFn)(void*);

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
  DeleterFn deleter;
} pxs_Var;

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
typedef struct pxs_Var *(*pxs_Func)(struct pxs_Var *args, void *opaque);

typedef void *pxs_Opaque;

typedef void (*FreeMethod)(void *ptr);

/**
 * Type Helper for a pxs_Var
 * Use this instead of writing out pxs_Var*
 */
typedef struct pxs_Var *pxs_VarT;

/**
 * Function Type for Loading a file.
 */
typedef char *(*LoadFileFn)(const char *file_path);

/**
 * Function Type for writing a file.
 */
typedef void (*WriteFileFn)(const char *file_path, const char *contents);

/**
 * Type for DirHandle.
 *
 * Host owns memory.
 */
typedef struct pxs_DirHandle {
  /**
   * The Length of the array
   */
  uintptr_t length;
  /**
   * The array values
   */
  char **values;
} pxs_DirHandle;

/**
 * Function Type for reading a Dir.
 */
typedef struct pxs_DirHandle (*ReadDirFn)(const char *dir_path);

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
 * Execute some lua code. Will return a String, an empty string means that the
 * code executed succesffuly
 *
 * The result needs to be freed by calling `pxs_free_str`
 */
char *pxs_execlua(const char *code, const char *file_name);

/**
 * Execute some Python code. Will return a String, an empty string means that the code executed successfully.
 *
 * The result needs to be freed by calling `pxs_free_str`
 */
char *pxs_execpython(const char *code,
                     const char *file_name);

/**
 * Free the string created by the pixelscript library
 */
void pxs_freestr(char *string);

/**
 * Create a new pixelscript Module.
 */
struct pxs_Module *pxs_newmod(const char *name);

/**
 * Add a callback to a module.
 *
 * Pass in the modules pointer and callback paramaters.
 */
void pxs_addfunc(struct pxs_Module *module_ptr, const char *name, pxs_Func func, pxs_Opaque opaque);

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
 * This must be wrapped in a `pxs_var_object` before use within a callback. If setting to a variable, this is done automatically for you.
 */
struct pxs_PixelObject *pxs_newobject(pxs_Opaque ptr,
                                      FreeMethod free_method,
                                      const char *type_name);

/**
 * Add a callback to a object.
 */
void pxs_object_addfunc(struct pxs_PixelObject *object_ptr,
                        const char *name,
                        pxs_Func callback,
                        pxs_Opaque opaque);

/**
 * Add a object to a Module.
 *
 * This essentially makes it so that when constructing this Module, this object is instanced.
 *
 * Depending on the language, you may need to wrap the construction. For example lua:
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
 * # etc
 * ```
 *
 * In JS/easyjs:
 * ```js
 * let p = new Person("Jordan", 23);
 * ```
 */
void pxs_addobject(struct pxs_Module *module_ptr,
                   const char *name,
                   pxs_Func object_constructor,
                   pxs_Opaque opaque);

/**
 * Make a new Var string.
 */
struct pxs_Var *pxs_newstring(const char *str);

/**
 * Make a new Null var.
 */
struct pxs_Var *pxs_newnull(void);

/**
 * Make a new HostObject var.
 *
 * If not a valid pointer, will return null
 *
 * Transfers ownership
 */
struct pxs_Var *pxs_newhost(struct pxs_PixelObject *pixel_object);

/**
 * Create a new variable int. (i64)
 */
struct pxs_Var *pxs_newint(int64_t val);

/**
 * Create a new variable uint. (u64)
 */
struct pxs_Var *pxs_newuint(uint64_t val);

/**
 * Create a new variable bool.
 */
struct pxs_Var *pxs_newbool(bool val);

/**
 * Create a new variable float. (f64)
 */
struct pxs_Var *pxs_newfloat(double val);

/**
 * Call a function on a object, and use a Enum for runtime rather than a var.
 *
 * var is self.
 */
struct pxs_Var *pxs_object_callrt(enum pxs_Runtime runtime,
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
 *     Var* obj = argv[1];
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
 */
bool pxs_getbool(struct pxs_Var *var);

/**
 * Get a String
 *
 * DANGEROUS
 *
 * You have to free this memory by calling `pxs_free_str`
 */
char *pxs_getstring(struct pxs_Var *var);

/**
 * Get the pointer of the Host Object
 *
 * This is "potentially" dangerous.
 */
pxs_Opaque pxs_gethost(struct pxs_Var *var);

/**
 * Check if a variable is of a type.
 */
bool pxs_varis(struct pxs_Var *var, enum pxs_VarType var_type);

/**
 * Set a function for reading a file.
 *
 * This is used to load files via import, require, etc
 */
void pxs_set_filereader(LoadFileFn func);

/**
 * Set a function for writing a file.
 *
 * This is used to write files via pxs_json
 */
void pxs_set_filewriter(WriteFileFn func);

/**
 * Set a function for reading a directory.
 *
 * This is used to read a dir.
 */
void pxs_set_dirreader(ReadDirFn func);

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
struct pxs_Var *pxs_tostring(struct pxs_Var *runtime, struct pxs_Var *var);

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
 * Call the opaque pointer of a object based on it's idx from `pxs_getobject`
 * This should only be used when derefing a passed in argument.
 * For `self` use `pxs_listget(args, 1)` and `pxs_gethost`.
 */
pxs_Opaque pxs_host_fromidx(int32_t idx);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* PIXEL_SCRIPT_H */
