#ifndef PIXEL_SCRIPT_H
#define PIXEL_SCRIPT_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * This represents the variable type that is being read or created.
 */
enum VarType {
  Int32,
  Int64,
  UInt32,
  UInt64,
  String,
  Bool,
  Float32,
  Float64,
  /**
   * Lua (nil), Python (None), JS/easyjs (null)
   */
  Null,
  /**
   * Lua (Tree), Python (Class), JS/easyjs (Prototype)
   */
  Object,
  /**
   * Host object converted when created.
   * Lua (Tree), Python (object), JS/easyjs (Prototype think '{}')
   */
  HostObject,
};
typedef uint32_t VarType;

/**
 * Public enum for supported runtimes.
 */
typedef enum PixelScriptRuntime {
  Lua,
  Python,
  JavaScript,
  Easyjs,
} PixelScriptRuntime;

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
typedef struct Module Module;

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
typedef struct PixelObject PixelObject;

/**
 * The Variables actual value union.
 */
typedef union VarValue {
  int32_t i32_val;
  int64_t i64_val;
  uint32_t u32_val;
  uint64_t u64_val;
  char *string_val;
  bool bool_val;
  float f32_val;
  double f64_val;
  const void *null_val;
  void *object_val;
  int32_t host_object_val;
} VarValue;

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
 * - Objects (these are a more of a pseudo-type)
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
typedef struct Var {
  /**
   * A tag for the variable type.
   */
  VarType tag;
  /**
   * A value as a union.
   */
  union VarValue value;
} Var;

/**
 * Function reference used in C.
 *
 * argc: i32, The number of args.
 * argc: *const *mut Var, a C Array of args.
 * opaque: *mut c_void, opaque user data.
 *
 * Func handles it's own memory, so no need to free the *mut Var returned or the argvs.
 *
 * But if you use any Vars within the function, you will have to free them before the function returns.
 */
typedef struct Var *(*Func)(uintptr_t argc, struct Var **argv, void *opaque);

typedef void (*FreeMethod)(void *ptr);

/**
 * Current pixelscript version.
 */
uint32_t pixelscript_version(void);

/**
 * Initialize the PixelScript runtime.
 */
void pixelscript_initialize(void);

/**
 * Finalize the PixelScript runtime.
 */
void pixelscript_finalize(void);

/**
 * Add a variable to the __main__ context.
 * Gotta pass in a name, and a Variable value.
 */
void pixelscript_add_variable(const char *name, const struct Var *variable);

/**
 * Add a callback to the __main__ context.
 * Gotta pass in a name, Func, and a optionl *void opaque data type
 */
void pixelscript_add_callback(const char *name, Func func, void *opaque);

/**
 * Execute some lua code. Will return a String, an empty string means that the
 * code executed succesffuly
 *
 * The result needs to be freed by calling `pixelscript_free_str`
 */
char *pixelscript_exec_lua(const char *code, const char *file_name);

/**
 * Execute some Python code. Will return a String, an empty string means that the code executed successfully.
 *
 * The result needs to be freed by calling `pixelscript_free_str`
 */
char *pixelscript_exec_python(const char *code,
                              const char *file_name);

/**
 * Free the string created by the pixelscript library
 */
void pixelscript_free_str(char *string);

/**
 * Create a new pixelscript Module.
 */
struct Module *pixelscript_new_module(const char *name);

/**
 * Add a callback to a module.
 *
 * Pass in the modules pointer and callback paramaters.
 */
void pixelscript_module_add_callback(struct Module *module_ptr,
                                     const char *name,
                                     Func func,
                                     void *opaque);

/**
 * Add a Varible to a module.
 *
 * Pass in the module pointer and variable params.
 */
void pixelscript_module_add_variable(struct Module *module_ptr,
                                     const char *name,
                                     const struct Var *variable);

/**
 * Add a Module to a Module
 *
 * This transfers ownership.
 */
void pixelscript_module_add_module(struct Module *parent_ptr, struct Module *child_ptr);

/**
 * Add the module finally to the runtime.
 *
 * After this you can forget about the ptr since PM handles it.
 */
void pixelscript_add_module(struct Module *module_ptr);

/**
 * Optionally free a module if you changed your mind.
 */
void pixelscript_free_module(struct Module *module_ptr);

/**
 * Create a new object.
 *
 * This should only be used within a PixelScript function callback, or globally set to 1 variable.
 *
 * This must be wrapped in a `pixelscript_var_object` before use within a callback. If setting to a variable, this is done automatically for you.
 */
struct PixelObject *pixelscript_new_object(void *ptr,
                                           FreeMethod free_method);

/**
 * Add a callback to a object.
 */
void pixelscript_object_add_callback(struct PixelObject *object_ptr,
                                     const char *name,
                                     Func callback,
                                     void *opaque);

/**
 * Add a object globally.
 *
 * This works as a Tree/Class/Prototype depending on the language.
 *
 * This is essentially just a factory callback but with special linking process.
 */
void pixelscript_add_object(const char *name, Func callback, void *opaque);

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
void pixelscript_module_add_object(struct Module *module_ptr,
                                   const char *name,
                                   Func object_constructor,
                                   void *opaque);

/**
 * Make a new Var string.
 *
 * Does take ownership
 */
struct Var *pixelscript_var_newstring(char *str);

/**
 * Make a new Null var.
 */
struct Var *pixelscript_var_newnull(void);

/**
 * Make a new HostObject var.
 *
 * If not a valid pointer, will return null
 *
 * Transfers ownership
 */
struct Var *pixelscript_var_newhost_object(struct PixelObject *pixel_object);

/**
 * Create a new variable i32.
 */
struct Var *pixelscript_var_newi32(int32_t val);

/**
 * Create a new variable u32.
 */
struct Var *pixelscript_var_newu32(uint32_t val);

/**
 * Create a new variable i64.
 */
struct Var *pixelscript_var_newi64(int64_t val);

/**
 * Create a new variable u64.
 */
struct Var *pixelscript_var_newu64(uint64_t val);

/**
 * Create a new variable bool.
 */
struct Var *pixelscript_var_newbool(bool val);

/**
 * Create a new variable f32.
 */
struct Var *pixelscript_var_newf32(float val);

/**
 * Create a new variable f64
 */
struct Var *pixelscript_var_newf64(double val);

struct Var *pixelscript_object_call_rt(enum PixelScriptRuntime runtime,
                                       struct Var *var,
                                       const char *method,
                                       uintptr_t argc,
                                       struct Var **argv);

/**
 * Object call.
 *
 * All memory is borrowed.
 *
 * You can get the runtime from the first Var in any callback.
 *
 * Example
 * ```C
 *     // Inside a Var* method
 *     Var* obj = argv[1];
 *     Var name = pixelscript_object_call()
 * ```
 */
struct Var *pixelscript_object_call(struct Var *runtime,
                                    struct Var *var,
                                    const char *method,
                                    uintptr_t argc,
                                    struct Var **argv);

/**
 * Get a I32 from a var.
 */
int32_t pixelscript_var_get_i32(struct Var *var);

/**
 * Get a I64 from a var.
 */
int64_t pixelscript_var_get_i64(struct Var *var);

/**
 * Get a U32 from a var.
 */
uint32_t pixelscript_var_get_u32(struct Var *var);

/**
 * Get a U64
 */
uint64_t pixelscript_var_get_u64(struct Var *var);

/**
 * Get a F32
 */
float pixelscript_var_get_f32(struct Var *var);

/**
 * Get a F64
 */
double pixelscript_var_get_f64(struct Var *var);

/**
 * Get a Bool
 */
bool pixelscript_var_get_bool(struct Var *var);

/**
 * Get a String
 *
 * DANGEROUS
 *
 * You have to free this memory by calling `pixelscript_free_str`
 */
char *pixelscript_var_get_string(struct Var *var);

/**
 * Get the pointer of the Host Object
 *
 * This is "potentially" dangerous.
 */
void *pixelscript_var_get_host_object(struct Var *var);

/**
 * Get the IDX of the PixelObject
 */
int32_t pixelscript_var_get_object_idx(struct Var *var);

#endif  /* PIXEL_SCRIPT_H */
