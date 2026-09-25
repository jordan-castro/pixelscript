//! Pocketpy rust bindings. The whole point of this is to cut down on compile times and control the stack better.
//! I had thought about doing this for a while, because I dont need to full pocketpy API to do what I am doing.
//! And if I need to add anything else, well I can do just that.
//! 
//! This matches the current version in libs. (2.2.0)

// == GLOBALS ==

pub const PXSPYTHON_IS_DIR : core::ffi::c_int = -2;
pub const PXSPYTHON_NOT_FOUND : core::ffi::c_int = -1;

// == GLOBALS END ==

// == TYPES ==

#[repr(i32)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum py_PredefinedType {
    tp_nil = 0,
    tp_object = 1,
    tp_type,  // py_Type
    tp_int,
    tp_float,
    tp_bool,
    tp_str,
    tp_str_iterator,
    tp_list,            // c11_vector
    tp_tuple,           // N slots
    tp_list_iterator,   // 1 slot
    tp_tuple_iterator,  // 1 slot
    tp_slice,           // 3 slots (start, stop, step)
    tp_range,
    tp_range_iterator,
    tp_module,
    tp_function,
    tp_nativefunc,
    tp_boundmethod,  // 2 slots (self, func)
    tp_super,        // 1 slot + py_Type
    tp_BaseException,
    tp_Exception,
    tp_bytes,
    tp_bytes_iterator,
    tp_namedict,
    tp_locals,
    tp_code,
    tp_dict,
    tp_dict_iterator,  // 1 slot
    tp_property,       // 2 slots (getter + setter)
    tp_star_wrapper,   // 1 slot + int level
    tp_staticmethod,   // 1 slot
    tp_classmethod,    // 1 slot
    tp_NoneType,
    tp_NotImplementedType,
    tp_ellipsis,
    tp_generator,
    /* builtin exceptions */
    tp_SystemExit,
    tp_KeyboardInterrupt,
    tp_StopIteration,
    tp_SyntaxError,
    tp_RecursionError,
    tp_OSError,
    tp_PermissionError,
    tp_NotImplementedError,
    tp_TypeError,
    tp_IndexError,
    tp_ValueError,
    tp_RuntimeError,
    tp_TimeoutError,
    tp_ZeroDivisionError,
    tp_NameError,
    tp_UnboundLocalError,
    tp_AttributeError,
    tp_ImportError,
    tp_AssertionError,
    tp_KeyError,
}

/// A helper struct for `py_Name`.
#[repr(C)]
pub struct py_OpaqueName {
    // I directly copy what bindgen generates here.
    _unused: [u8; 0],
}

/// A pointer that represents a python identifier. For fast name resolution.
pub type py_Name = *mut py_OpaqueName;

/// An integer that represents a python type. `0` is invalid.
pub type py_Type = i16;

pub type py_CFunction = unsafe extern "C" fn(argc: core::ffi::c_int, argv: py_StackRef) -> bool;

#[repr(C)]
#[derive(Copy, Clone)]
/// Union for `py_TValue`
union py_TValue_Union {
    _i64: i64,
    _f64: f64,
    _bool: bool,
    _cfunc: py_CFunction,
    _obj: *mut core::ffi::c_void,
    _ptr: *mut core::ffi::c_void,
    _chars: [core::ffi::c_char; 16]
}

/// A opaque type that represents a python object. You cannot access its members directly.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct py_TValue {
    _type: py_Type,
    is_ptr: bool,
    extra: core::ffi::c_int,
    _data: py_TValue_Union
}

/// A 64-bit integer type. Corresponds to `int` in python.
pub type py_i64 = i64;
/// A 64-bit floating-point type. Corresponds to `float` in python.
pub type py_f64 = f64;

/// A generic reference to a python object.
pub type py_Ref = *mut py_TValue;
/// A reference which has the same lifespan as the python object.
pub type py_ObjectRef = *mut py_TValue;
/// A global reference which has the same lifespan as the VM.
pub type py_GlobalRef = *mut py_TValue;
/// A specific location in the value stack of the VM.
pub type py_StackRef = *mut py_TValue;
/// An item reference to a container object. It invalidates when the container is modified.
pub type py_ItemRef = *mut py_TValue;
/// An output reference for returning a value. Only use this for function arguments.
pub type py_OutRef = *mut py_TValue;

pub type import_file_func = unsafe extern "C" fn (path: *const core::ffi::c_char, data_size: *mut core::ffi::c_int) -> *mut core::ffi::c_char;

/// A struct contains the callbacks of the VM.
#[repr(C)]
pub struct py_Callbacks {
    pub importfile: Option<import_file_func>,
    lazyimport: Option<unsafe extern "C" fn(*const core::ffi::c_char) -> py_GlobalRef>,
    print: Option<unsafe extern "C" fn(*const core::ffi::c_char)>,
    flush: Option<unsafe extern "C" fn()>,
    getchr: Option<unsafe extern "C" fn() -> core::ffi::c_int>,
    gc_mark: Option<unsafe extern "C" fn(extern "C" fn(val: py_Ref, ctx: *mut core::ffi::c_void), ctx: *mut core::ffi::c_void)>,
    displayhook: Option<unsafe extern "C" fn(val: py_Ref) -> bool>
}

/// Python compiler modes.
/// + `EXEC_MODE`: for statements.
/// + `EVAL_MODE`: for expressions.
/// + `SINGLE_MODE`: for REPL or jupyter notebook execution.
/// + `RELOAD_MODE`: for reloading a module without allocating new types if possible.
#[repr(i32)]
pub enum py_CompileMode {
    EXEC_MODE = 0,
    EVAL_MODE = 1,
    SINGL_MODE = 2,
    RELOAD_MODE = 3
}

// == TYPES END ==

// == FUNCTIONS ==

unsafe extern "C" {
    /// Initialize pocketpy and the default VM.
    pub fn py_initialize();
    /// Finalize pocketpy and free all VMs. This opearation is irreversible.
    /// After this call, you cannot use any function from this header anymore.
    pub fn py_finalize();
    /// Get the current VM index.
    pub fn py_currentvm() -> core::ffi::c_int;
    /// Switch to a VM.
    /// @param index index of the VM ranging from 0 to 16 (exclusive). `0` is the default VM.
    pub fn py_switchvm(index: core::ffi::c_int);
    /// Reset the current VM.
    pub fn py_resetvm();
    /// Reset All VMs.
    pub fn py_resetallvm();
    /// Setup the callbacks for the current VM.
    pub fn py_callbacks() -> *mut py_Callbacks;
    /// Invoke the garbage collector.
    pub fn py_gc_collect() -> core::ffi::c_int;
    /// Wrapper for `PK_FREE(ptr)`.
    pub fn py_free(ptr: *mut core::ffi::c_void);
    /// Compile a source string into a code object.
    /// Use python's `exec()` or `eval()` to execute it.
    pub fn py_compile(source: *const core::ffi::c_char, filename: *const core::ffi::c_char, mode: py_CompileMode, is_dynamic: bool) -> bool;
    /// Run a source string.
    /// @param source source string.
    /// @param filename filename (for error messages).
    /// @param mode compile mode. Use `EXEC_MODE` for statements `EVAL_MODE` for expressions.
    /// @param module target module. Use NULL for the main module.
    /// @return `true` if the execution is successful or `false` if an exception is raised.
    pub fn py_exec(source: *const core::ffi::c_char, filename: *const core::ffi::c_char, mode: py_CompileMode, module: py_Ref) -> bool;
    /// Create an `int` object.
    pub fn py_newint(source: py_OutRef, value: py_i64);
    /// Create a `float` object.
    pub fn py_newfloat(source: py_OutRef, value: py_f64);
    /// Create a `bool` object.
    pub fn py_newbool(source: py_OutRef, value: bool);
    /// Create a `str` object from a null-terminated string (utf-8).
    pub fn py_newstr(source: py_OutRef, value: *const core::ffi::c_char);
    /// Create a `None` object.
    pub fn py_newnone(source: py_OutRef);
    /// Convert a null-terminated string to a name.
    pub fn py_name(str: *const core::ffi::c_char) ->  py_Name;
    /// Bind a function to the object via "argc-based" style.
    /// @param obj the target object.
    /// @param name name of the function.
    /// @param f function to bind.
    pub fn py_bindfunc(obj: py_Ref ,  name: *const core::ffi::c_char, f: Option<py_CFunction>);
    /// Convert an `int` object in python to `int64_t`.
    pub fn py_toint(pref: py_Ref) -> py_i64;
    /// Convert a `float` object in python to `double`.
    pub fn py_tofloat(pref: py_Ref) -> py_f64;
    /// Convert a `bool` object in python to `bool`.
    pub fn py_tobool(pref: py_Ref) -> bool;
    /// Convert a `str` object in python to null-terminated string.
    pub fn py_tostr(pref: py_Ref) -> *const core::ffi::c_char;
    /// Get the type of the object.
    pub fn py_typeof(pref: py_Ref) -> py_Type;
    /// Get the current `module` object where the code is executed.
    /// Return `NULL` if not available.
    pub fn py_inspect_currentmodule() -> py_GlobalRef;
    /// Get the last return value.
    /// Please note that `py_retval()` cannot be used as input argument.
    pub fn py_retval() -> py_GlobalRef;
    /// Get an item from the object's `__dict__`.
    /// Return `NULL` if not found.
    pub fn py_getdict(pref: py_Ref, name: py_Name) -> py_ItemRef;
    /// Get variable in the `builtins` module.
    pub fn py_getbuiltin(name: py_Name) -> py_ItemRef;
    /// Get variable in the `__main__` module.
    pub fn py_getglobal(name: py_Name) -> py_ItemRef;
    /// Push the object to the stack.
    pub fn py_push(pref: py_Ref);
    /// Push a `nil` object to the stack.
    pub fn py_pushnil();
    /// Pop an object from the stack.
    pub fn py_pop();
    /// Get a temporary variable from the stack.
    pub fn py_pushtmp() -> py_StackRef;
    /// Call a callable object via pocketpy's calling convention.
    /// You need to prepare the stack using the following format:
    /// `callable, self/nil, arg1, arg2, ..., k1, v1, k2, v2, ...`.
    /// `argc` is the number of positional arguments excluding `self`.
    /// `kwargc` is the number of keyword arguments.
    /// The result will be set to `py_retval()`.
    /// The stack size will be reduced by `2 + argc + kwargc * 2`.
    pub fn py_vectorcall(argc: u16, kwargc: u16) -> bool;
    /// Call a type to create a new instance.
    pub fn py_tpcall(t: py_Type, argc: core::ffi::c_int, argv: py_Ref) -> bool;
    /// Python equivalent to `len(val)`.
    pub fn py_len(val: py_Ref) -> bool;
    /// Python equivalent to `getattr(self, name)`.
    pub fn py_getattr( s: py_Ref, name: py_Name ) -> bool;
    /// Python equivalent to `setattr(self, name, val)`.
    pub fn py_setattr(s: py_Ref, name: py_Name, val: py_Ref ) -> bool;
    /// Python equivalent to `delattr(self, name)`.
    pub fn py_delattr(s: py_Ref,  name: py_Name) -> bool;
    /// Python equivalent to `self[key]`.
    pub fn py_getitem(s: py_Ref, key: py_Ref ) -> bool;
    /// Python equivalent to `self[key] = val`.
    pub fn py_setitem(s: py_Ref , key : py_Ref , val: py_Ref ) -> bool;
    /// Python equivalent to `del self[key]`.
    pub fn py_delitem(s: py_Ref , key: py_Ref ) -> bool;
    /// Get a module by path.
    pub fn py_getmodule(path: *const core::ffi::c_char) -> py_GlobalRef;
    /// Create a new module.
    pub fn py_newmodule(path: *const core::ffi::c_char) -> py_GlobalRef;
    /// Clear the unhandled exception.
    /// @param p0 the unwinding point. Use `NULL` if not needed.
    pub fn py_clearexc(p0: py_StackRef);
    /// Format the unhandled exception and return a null-terminated string.
    /// The returned string should be freed by the caller.
    pub fn py_formatexc() -> *mut core::ffi::c_char;
    /// Raise an exception object. Always return false.
    pub fn py_raise(exc: py_Ref) -> bool;
    /// Override for the pocketpy.callbacks.import function.
    pub fn pxspython_import(path: *const core::ffi::c_char, size: *mut core::ffi::c_int) -> *mut core::ffi::c_char;
    /// Create an empty `list`.
    pub fn py_newlist(oref: py_OutRef);
    pub fn py_list_append(s: py_Ref, val: py_Ref);
    /// Create an empty `dict`.
    pub fn py_newdict(oref: py_OutRef);
    /// -1: error, 0: not found, 1: found
    pub fn py_dict_getitem(s: py_Ref, k: py_Ref) -> core::ffi::c_int;
    /// true: success, false: error
    pub fn py_dict_setitem(s: py_Ref, key: py_Ref , val: py_Ref ) -> bool;
    /// -1: error, 0: not found, 1: found (and deleted)
    pub fn py_dict_delitem(s: py_Ref, key: py_Ref) -> core::ffi::c_int;
    /// -1: error, 0: not found, 1: found
    pub fn py_dict_getitem_by_int(s: py_Ref, key: py_i64) -> core::ffi::c_int;
    /// -1: error, 0: not found, 1: found (and deleted)
    pub fn py_dict_delitem_by_int(s: py_Ref, key: py_i64) -> core::ffi::c_int;

}

// == FUNCTIONS END ==