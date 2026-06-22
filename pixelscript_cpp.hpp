// C++ RAII style wrappers
#pragma once

// Expects this to be a SYSTEM library
#include <pixelscript.h>
#include <pixelscript_m.h>

#include <string>
#include <vector>
#include <cstring>
#include <cstdlib>
#include <optional>

// The pixel script namespace
namespace pxs {
    // A Var
    class Var {
        // Runtime this Var belongs to
        pxs_Var* rt;
        // Ptr to pxs_Var
        pxs_Var* ptr;
        // Is this a owned ptr? i.e. we need to free it when deleted
        bool owned;

    public:
        Var() : rt(nullptr), ptr(pxs_newnull()), owned(true) {}
        Var(pxs_Var* ptr) : rt(nullptr), ptr(ptr), owned(false) {}
        Var(pxs_Var* ptr, bool owned) : rt(nullptr), ptr(ptr), owned(owned) {}
        Var(pxs_Var* rt, pxs_Var* ptr, bool owned=false) : rt(rt), ptr(ptr), owned(owned) {}
        ~Var() {
            if (owned && ptr != nullptr) {
                pxs_freevar(ptr);
            }
        }

        Var(Var&& other) noexcept : rt(other.rt), ptr(other.ptr), owned(other.owned) {
            other.ptr = nullptr;
            other.owned = false;
        }

        Var(const Var& other) = delete;
        Var& operator=(const Var& other) = delete;

        Var& operator=(Var&& other) noexcept {
            if (this != &other) {
                rt = other.rt;
                ptr = other.ptr;
                owned = other.owned;

                other.rt = nullptr;
                other.ptr = nullptr;
                other.owned = false;
            }

            return *this;
        }

        // Gets a variable from the method paramaters.
        // assumes that index - 1 is passed. I.e. runtime is ignored.
        // runtime is automatiacally set via args(0)
        [[nodiscard]] static Var from_args(pxs_Var* args, int index, bool owned=false) {
            return Var(pxs_listget(args, 0), pxs_listget(args, index + 1), owned);
        }

        // Creates a list for passing into pxs functions.
        // Literally does:
        // `pxs_newlist() and pxs_listadd(pxs_newcopy(rt))`
        [[nodiscard]] static Var new_args(pxs_Var* rt, bool owned=true) {
            auto var =  Var(rt, pxs_newlist(), owned);

            // add rt copy to list
            var.add(pxs_newcopy(rt));

            return var;
        }

        // Creates a list for passing into pxs factories
        // Lietrally does:
        // `pxs_newlist()` and returns it.
        [[nodiscard]] static Var new_factory_args() {
            auto var = Var(nullptr, pxs_newlist());
            return var;
        }

        // Create a new list
        [[nodiscard]] static Var new_list() {
            return Var(nullptr, pxs_newlist());
        }

        // Create a Var with the same rt
        [[nodiscard]] Var with_rt(pxs_Var* val, bool owned=false) const {
            return Var(rt, val, owned);
        }

        // Create a null Var.
        [[nodiscard]] Var null(bool owned=false) const {
            return with_rt(pxs_newnull(), owned);
        }

        // Create a null Var without a runtime.
        [[nodiscard]] static Var new_null(bool owned=false) {
            return pxs::Var(pxs_newnull(), nullptr, owned);
        }

        // Create a Exception Var without a runtime.
        [[nodiscard]] static Var new_exception(const std::string& msg) {
            return pxs::Var(pxs_newexception(msg.c_str()));
        }

        // Get access to raw ptr.
        [[nodiscard]] pxs_Var* raw() const {
            return ptr;
        }

        // Check variable type
        bool is(pxs_VarType T) const {
            return pxs_varis(ptr, T);
        }

        // Get int val. Will return -1 if not a int or float.
        [[nodiscard]] int64_t get_int() const {
            return pxs_getint(ptr);
        }

        // Get uint val. Will rturn 0 if not a uin, int or float.
        [[nodiscard]] uint64_t get_uint() const {
            return pxs_getuint(ptr);
        }

        // Get string val. Will return "" if not a valid string.
        // Will also free the memoery automatically
        [[nodiscard]] std::string get_string() const {
            if (!is(pxs_String) && !is(pxs_Exception)) {
                return std::string("");
            }

            // Ok lets do it!
            char* val = pxs_getstring(ptr);
            if (val == nullptr) {
                return std::string("");
            }

            std::string res = std::string(val);
            pxs_freestr(val);
            return res;
        }

        // Get float val. Will return -1.0f if not a int or float
        [[nodiscard]] float get_float() const {
            return pxs_getfloat(ptr);
        }

        // Get bool val. Will return false if not bool.
        [[nodiscard]] bool get_bool() const {
            if (!is(pxs_Bool)) {
                return false;
            }
            return pxs_getbool(ptr);
        }

        // Add a pxs_Var or Var to this Var as a list.
        // This does NOT copy.
        void add(pxs_Var* arg) const {
            if (arg == nullptr || !is(pxs_List)) {
                return;
            }

            pxs_listadd(ptr, arg);
        }

        // Add a `Var` with shallow copy in place.
        void add_shallow(const Var& arg) const {
            if (!is(pxs_List)) {
                return;
            }

            pxs_listadd(ptr, pxs_new_shallowcopy(arg.raw()));
        }

        // When adding a Var, copy is assumed.
        void add(const Var& arg) const {
            if (!is(pxs_List)) {
                return;
            }

            pxs_listadd(ptr, pxs_newcopy(arg.raw()));
        }

        // Get this var as a C++ object. HostObject
        template<typename T>
        T* get_object() const {
            pxs_Opaque obj = pxs_gethost(rt, raw());

            if (obj == nullptr) {
                return nullptr;
            }

            // Deref
            return static_cast<T*>(obj);
        }

        // Call this variable as a function
        [[nodiscard]] Var as_func(const std::vector<Var> &args) const {
            // Get raw
            Var list = Var(rt, pxs_newlist());
            for (auto& a : args) {
                list.add(a);
            }
            
            // Now we can call as function
            auto res = pxs_varcall(rt, raw(), list.raw());
            if (res == nullptr) {
                return null(true);
            } else {
                return Var(rt, res, true);
            }
        }

        // Call a method on this object. Not a HostObject.
        // Result will automatically be freed when out of scope. So copy it if you need it.
        [[nodiscard]] Var call(const std::string& method, const std::vector<Var>& args) const {
            if (is(pxs_HostObject)) {
                return null(true);
            }
            // Get raw
            Var list = Var(rt, pxs_newlist());
            for (auto& a : args) {
                list.add(a);
            }

            auto res = pxs_objectcall(rt, ptr, method.c_str(), list.raw());
            pxs_Var* result = nullptr;
            if (res == nullptr) {
                result = pxs_newnull();
            } else {
                result = res;
            }

            return Var(rt, result, true);
        }

        // Easily return a copy of the current Var.
        [[nodiscard]] Var copy_owned() const {
            return Var(rt, pxs_newcopy(ptr), true);
        }

        // Copy current var but dont own
        [[nodiscard]] Var copy() const {
            return Var(rt, pxs_new_shallowcopy(ptr), true);
        }

        // For lists, get at specific index.
        [[nodiscard]] Var list_get(int index) const {
            // Check if ptr is a list
            if (!is(pxs_List)) {
                return null(true);
            }

            // get item
            auto item = pxs_listget(ptr, index);
            if (item == nullptr) {
                return null(true);
            } else {
                return with_rt(item, false).copy_owned();
            }
        }

        // Call a getter on object.
        [[nodiscard]] Var get(const std::string& key) const {
            if (!is(pxs_Object)) {
                return null(true);
            }

            return Var(rt, pxs_objectget(rt, raw(), key.c_str()), true);
        }

        // Get length of list var
        int list_len() const {
            if (!is(pxs_List)) {
                return -1;
            }

            return pxs_listlen(raw());
        }

        // Debug a `pxs_Var`
        std::string debug() const {
            // Get string
            auto str = pxs_debugvar(raw());
            std::string res(str);
            pxs_freestr(str);
            return res;
        }

        static std::string debug(pxs_VarT var) {
            return pxs::Var(var).debug();
        }

        // Set owned
        void set_owned(bool val) {
            owned = val;
        }

        // Get all the items of a list as objects.
        template<typename T>
        std::vector<T> list_get_objects() const {
            std::vector<T> res;
            if (!is(pxs_List)) {
                return res;
            }

            for (int i = 0; i < list_len(); i++) {
                auto it = list_get(i).get_object<T>();
                if (!it) {
                    continue;
                }
                res.push_back(T(*it));
            }

            return res;
        }

        // Convert current variable to string.
        std::string to_string() const {
            auto str_res = Var(pxs_tostring(rt, pxs_new_shallowcopy(ptr)));
            str_res.set_owned(true);
            return str_res.get_string();
        }

        // Run a compiled object
        [[nodiscard]] Var run(pxs_VarT local_scope) const {
            auto res = pxs_execobject(pxs_new_shallowcopy(raw()), local_scope);
            return with_rt(res);
        }
    };

    // Call a function using pxs_call
    // Result is owned.
    [[nodiscard]] inline Var call(pxs_Runtime runtime, const std::string& name, const pxs::Var& args) {
        // Check args is list
        if (!args.is(pxs_List)) {
            // Must be a list!
            return Var::new_exception(std::string("`args` in call is not list but is, ") + args.debug());
        }
        // Convert runtime into ptr
        auto rt = pxs_newint(runtime);
        // Call
        auto res = pxs_call(rt, name.c_str(), args.copy().raw());
        return pxs::Var(rt, res, true);
    }

    // Wrapper around `pxs_PixelObject`.
    class Object {
        pxs_PixelObject* obj;
        char* type;
        std::vector<char*> method_names;
    public:
        Object(void* ptr, pxs_DeleterFn deleter, const std::string& type) {
            this->type = strdup(type.c_str());
            this->obj = pxs_newobject(ptr, deleter, this->type);
            this->method_names = {};
        }

        ~Object() {
            if (this->type != nullptr) {
                free(this->type);
            }

            for (auto ptr : this->method_names) {
                free(ptr);
            }
        }

        // Turn this into a HostObject variable
        [[nodiscard]] Var make() {
            auto var = Var(pxs_newnull(), pxs_newhost(obj));
            obj = nullptr;

            return var;
        }

        // Add a method to the object
        void add_method(const std::string& method_name, pxs_Func func, bool use_id=true) {
            char* name = strdup(method_name.c_str());
            method_names.push_back(name);

            if (use_id) {
                pxs_object_addfunc(obj, name, func);
            } else {
                pxs_object_add_reffunc(obj, name, func);
            }
        }

        // Add a str method to the object. Will add for all runtimes supporting.
        void add_str_method(pxs_Runtime runtime, pxs_Func func) {
            if (runtime == pxs_Runtime::pxs_Python) {
                // Python
                add_method("__str__", func);
            } else if (runtime == pxs_Runtime::pxs_Lua) {
                // Lua
                add_method("__tostring", func);
            } else if (runtime == pxs_Runtime::pxs_JavaScript) {
                // JavaScript
                add_method("toString", func);
            }
        }

        // Add a str method to the object. Will add for all runtimes.
        void add_str_method(pxs_Func func) {
            add_str_method(pxs_Python, func);
            add_str_method(pxs_Lua, func);
            add_str_method(pxs_JavaScript, func);
        }

        // Add a property
        void add_property(const std::string& name, pxs_Func func) {
            pxs_object_addprop(obj, name.c_str(), func);    
        }
    };

    // Get runtime from int
    inline pxs_Runtime runtime_from_int(int val) {
        if (val == 0) {
            return pxs_Lua;
        } else if (val == 1) {
            return pxs_Python;
        } else if (val == 2) {
            return pxs_JavaScript;
        } else {
            return pxs_Python;
        }
    }

    // Get runtime from pxs_Var
    inline pxs_Runtime runtime_from_var(pxs_VarT var) {
        return runtime_from_int(pxs_getint(var));
    }

    // Get runtime from Var
    inline pxs_Runtime runtime_from_var(const Var& var) {
        return runtime_from_var(var.raw());
    }

    // // Get runtime from file extension
    // inline std::optional<pxs_Runtime> runtime_from_file_extension(const std::string& file_path) {
    //     if (utils::ends_with(file_path, ".lua")) {
    //         return pxs_Lua;
    //     } else if (utils::ends_with(file_path, ".py")) {
    //         return pxs_Python;
    //     } else if (utils::ends_with(file_path, ".js")) {
    //         return pxs_JavaScript;
    //     } else {
    //         return std::nullopt;
    //     }
    // }

    // Compile a script into a code object. Runtime is not inferred.
    inline Var compile(pxs_Runtime runtime, const std::string& code, pxs_VarT global_scope) {
        auto res = pxs_compile(runtime, code.c_str(), global_scope);
        return pxs::Var(pxs_newint(static_cast<int>(runtime)), res);
    }

    // // Compile a script file into a code object. Runtime is inferred.
    // inline Var compile(const std::string& file_path, pxs_VarT global_scope) {
    //     // Check runtime from file path
    //     auto runtime = runtime_from_file_extension(file_path);
    //     if (!runtime.has_value()) {
    //         return Var::new_exception("Runtime not found for file").raw();
    //     }
    //     // Get value
    //     auto rt = runtime.value();

    //     // Check file contents
    //     auto contents = readFile(file_path.c_str());

    //     if (contents.empty()) {
    //         return Var::new_exception("File path is empty").raw();
    //     }

    //     return compile(rt, contents, global_scope);
    // }

    inline std::string string_type(pxs_VarType var_type) {
        switch(var_type) {
            case pxs_Int64:
                return "Int";
            case pxs_UInt64:
                return "UInt";
            case pxs_Float64:
                return "Float";
            case pxs_Bool:
                return "Bool";
            case pxs_String:
                return "String";
            case pxs_Null:
                return "Null";
            case pxs_Object:
                return "Object";
            case pxs_HostObject:
                return "HostObject";
            case pxs_List:
                return "List";
            case pxs_Function:
                return "Function";
            case pxs_Factory:
                return "Factory";
            case pxs_Exception:
                return "Exception";
            case pxs_Map:
                return "Map";
            default:
                return "Unkown"; 
        }
    }
    inline std::string string_type(pxs_VarT var) {
        auto v = pxs::Var(var);
        return v.debug();
    }
};

// Useful macros for PXS interop

// If argc is not equal to it will return exception
#define PXS_ARGC_EQ(expected) if (PXS_ARGC() != expected) return pxs::Var::new_exception(std::string("Expected ") + std::to_string(expected) + std::string(" args. Found ") + std::to_string(PXS_ARGC())).raw()

#define PXS_ARGC_GT(expected) if (PXS_ARGC() < expected) return pxs::Var::new_exception(std::string("Expected at least ") + std::to_string(expected) + std::string(" args. Found ") + std::to_string(PXS_ARGC())).raw()

#define PXS_ARGC_LT(expected) if (PXS_ARGC() > expected) return pxs::Var::new_exception(std::string("Expected at most ") + std::to_string(expected) + std::string(" args. Found ") + std::to_string(PXS_ARGC())).raw()

#define PXS_ARG_IS_TYPE(arg, expected) if (!pxs_varis(arg, expected)) return pxs::Var::new_exception(std::string("Expected " + pxs::string_type(expected) + " but found " + pxs::string_type(arg))).raw();


namespace pxs::type {
    // pixelscript does not know what a HostObject type is. It is just a void* passed around the host to the caller.
    // So to enforce that what we are receiving is correct. We need to attach a "TYPE" to it. Without the type, UB is possible.
    class HWrapper {
        pxs_Opaque data;
        pxs_DeleterFn deleter;
        // -1 means unkown/unset.
        int32_t type_tag = -1;

        public:
            HWrapper(pxs_Opaque data, pxs_DeleterFn deleter, int32_t tt) : data(data), deleter(deleter), type_tag(tt) {}
            ~HWrapper() {
                if (this->data != nullptr) {
                    this->deleter(this->data);
                    this->data = nullptr;
                    this->type_tag = -1;
                }
            }

            template<typename T>
            static T* get(const pxs::Var& var, int32_t expected_type) {
                auto wrapper = var.get_object<HWrapper>();
                if (!wrapper) {
                    return nullptr;
                }

                // Check type match
                if (wrapper->type_tag != expected_type) {
                    return nullptr;
                }

                return static_cast<T*>(wrapper->data);
            }
    };
};
