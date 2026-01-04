use shared::{func::Func, var::Var};
use std::{
    ffi::{CStr, CString, c_char, c_void},
    ptr,
    sync::{Arc, Mutex, MutexGuard, OnceLock},
};

use crate::{
    lua::LuaScripting,
    shared::{PixelScript, PtrMagic, func::get_function_lookup, module::Module, object::{FreeMethod, PixelObject, get_object_lookup}},
};

pub mod shared;

#[cfg(feature = "lua")]
pub mod lua;

/// Macro to wrap features
macro_rules! with_feature {
    ($feature:expr, $logic:block) => {
        #[cfg(feature=$feature)]
        {
            $logic
        }
    };
}

/// Convert a borrowed C string (const char *) into a Rust &str.
macro_rules! borrow_string {
    ($cstr:expr) => {{
        if $cstr.is_null() {
            ""
        } else {
            unsafe {
                let c_str = CStr::from_ptr($cstr);
                c_str.to_str().unwrap_or("")
            }
        }
    }};
}

/// Convert a owned C string (i.e. owned by us now.) into a Rust String.
///
/// The C memory will be freed automatically, and you get a nice clean String!
macro_rules! own_string {
    ($cstr:expr) => {{
        if $cstr.is_null() {
            String::new()
        } else {
            let owned_string = unsafe { CString::from_raw($cstr) };

            owned_string
                .into_string()
                .unwrap_or_else(|_| String::from("Invalid UTF-8"))
        }
    }};
}

/// Create a raw string from &str.
///
/// Remember to FREE THIS!
macro_rules! create_raw_string {
    ($rstr:expr) => {{ CString::new($rstr).unwrap().into_raw() }};
}

macro_rules! assert_initiated {
    () => {{
        unsafe {
            if !IS_INIT {
                panic!("Pixel Script library is not initialized.");
            }
        }
    }};
}

/// Is initialized?
static mut IS_INIT: bool = false;
/// Is killed?
static mut IS_KILLED: bool = false;

/// Current pixelscript version.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_version() -> u32 {
    0x00010000 // 1.0.0
}

/// Initialize the PixelScript runtime.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_initialize() {
    unsafe {
        if IS_KILLED {
            panic!("Once finalized, PixelScript can not be initalized again.");
        }
        if !IS_INIT {
            with_feature!("lua", {
                LuaScripting::start(); 
            });
        }
        IS_INIT = true;
    }
}

/// Finalize the PixelScript runtime.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_finalize() {
    assert_initiated!();

    unsafe {
        if IS_KILLED {
            panic!("Can not finalize the runtime twice.");
        }
        IS_KILLED = true;
    }

    // Drop function lookup
    get_function_lookup().function_hash.clear();
    // Drop object lookup
    get_object_lookup().object_hash.clear();

    with_feature!("lua", {
        LuaScripting::stop();
    });
}

/// Add a variable to the __main__ context.
/// Gotta pass in a name, and a Variable value.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_variable(name: *const c_char, variable: &Var) {
    assert_initiated!();
    // Get string as rust.
    let r_str = borrow_string!(name);
    if r_str.is_empty() {
        return;
    }

    // Add variable to lua context
    with_feature!("lua", {
        LuaScripting::add_variable(r_str, variable);
    });
}

/// Add a callback to the __main__ context.
/// Gotta pass in a name, Func, and a optionl *void opaque data type
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_callback(name: *const c_char, func: Func, opaque: *mut c_void) {
    assert_initiated!();

    // Get rust name
    let name_str = borrow_string!(name);
    if name_str.is_empty() {
        return;
    }

    // Add Function to lua context
    with_feature!("lua", {
        LuaScripting::add_callback(name_str, func, opaque);
    });
}

/// Execute some lua code. Will return a String, an empty string means that the
/// code executed succesffuly
///
/// The result needs to be freed by calling `pixelscript_free_str`
#[unsafe(no_mangle)]
#[cfg(feature = "lua")]
pub extern "C" fn pixelscript_exec_lua(
    code: *const c_char,
    file_name: *const c_char,
) -> *const c_char {
    assert_initiated!();
    // First convert code and file_name to rust strs
    let code_str = borrow_string!(code);
    if code_str.is_empty() {
        return create_raw_string!("Code is empty");
    }
    let file_name_str = borrow_string!(file_name);
    if file_name_str.is_empty() {
        return create_raw_string!("File name is empty");
    }

    // Execute and get result
    let result = lua::execute(code_str, file_name_str);

    create_raw_string!(result)
}

/// Free the string created by the pixelscript library
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_free_str(string: *mut c_char) {
    assert_initiated!();
    if !string.is_null() {
        unsafe {
            // Let the string go out of scope to be dropped
            let _ = CString::from_raw(string);
        }
    }
}

/// Create a new pixelscript Module.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_new_module(name: *const c_char) -> *mut Module {
    assert_initiated!();
    if name.is_null() {
        return ptr::null_mut();
    }
    let name_str = borrow_string!(name);

    Module::new(name_str.to_owned()).into_raw()
}

/// Add a callback to a module.
///
/// Pass in the modules pointer and callback paramaters.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_module_add_callback(
    module_ptr: *mut Module,
    name: *const c_char,
    func: Func,
    opaque: *mut c_void,
) {
    assert_initiated!();
    if module_ptr.is_null() {
        return;
    }

    if name.is_null() {
        return;
    }

    // Get actual data
    let module = unsafe { Module::from_borrow(module_ptr) };
    let name_str = borrow_string!(name);

    // Now add callback
    module.add_callback(name_str, func, opaque);
}

/// Add a Varible to a module.
///
/// Pass in the module pointer and variable params.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_module_add_variable(
    module_ptr: *mut Module,
    name: *const c_char,
    variable: &Var,
) {
    assert_initiated!();
    if module_ptr.is_null() {
        return;
    }

    if name.is_null() {
        return;
    }

    let module = unsafe { Module::from_borrow(module_ptr) };
    let name_str = borrow_string!(name);

    // Now add variable
    module.add_variable(name_str, variable);
}

/// Add a Module to a Module
///
/// This transfers ownership.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_module_add_module(parent_ptr: *mut Module, child_ptr: *mut Module) {
    assert_initiated!();
    if parent_ptr.is_null() || child_ptr.is_null() {
        return;
    }

    let parent = unsafe { Module::from_borrow(parent_ptr) };
    // Own child
    let child = Module::from_raw(child_ptr);

    parent.add_module(child);

    // Child is now owned by parent
}

/// Add the module finally to the runtime.
///
/// After this you can forget about the ptr since PM handles it.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_module(module_ptr: *mut Module) {
    assert_initiated!();
    if module_ptr.is_null() {
        return;
    }

    let module = Arc::new(Module::from_raw(module_ptr));

    // LUA
    with_feature!("lua", {
        LuaScripting::add_module(Arc::clone(&module));
    });

    // Module gets dropped here, and that is good!
}

/// Optionally free a module if you changed your mind.
#[unsafe(no_mangle)]
pub extern "C" fn pixelmods_free_module(module_ptr: *mut Module) {
    assert_initiated!();

    if module_ptr.is_null() {
        return;
    }

    let _ = Module::from_raw(module_ptr);
}

/// Create a new object.
/// 
/// This should only be used within a PixelScript function callback, or globally set to 1 variable.
/// 
/// This must be wrapped in a `pixelscript_var_object` before use within a callback. If setting to a variable, this is done automatically for you.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_new_object(ptr: *mut c_void, free_method: FreeMethod) -> *mut PixelObject {
    assert_initiated!();
    if ptr.is_null() {
        return ptr::null_mut();
    }

    PixelObject::new(ptr, free_method).into_raw()
}

/// Add a callback to a object.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_object_add_callback(object_ptr: *mut PixelObject, name: *const c_char, callback: Func, opaque: *mut c_void) {
    assert_initiated!();

    if object_ptr.is_null() || name.is_null() {
        return;
    }

    // Borrow ptr
    let object_borrow = unsafe{PixelObject::from_borrow(object_ptr)};
    let name_borrow = borrow_string!(name);
    object_borrow.add_callback(name_borrow, callback, opaque);
}

/// Add a object as a variable.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_object_variable(name: *const c_char, object_ptr: *mut PixelObject) {
    assert_initiated!();

    if name.is_null() || object_ptr.is_null() {
        return;
    }

    // Own the pointer
    let pixel_object = Arc::new(PixelObject::from_raw(object_ptr));
    let name_borrow = borrow_string!(name);

    with_feature!("lua", {
        LuaScripting::add_object_variable(name_borrow, Arc::clone(&pixel_object));
    });

    // Drops original object? NO because they live within the lookup!
}

/// Add a object to a Module.
/// 
/// This essentially makes it so that when constructing this Module, this object is instanced.
/// 
/// Depending on the language, you may need to wrap the construction. For example lua:
/// ```lua
/// // Let's say we have a object "Person"
/// local p = Person("Jordan", 23)
/// p.set_name("Jordan Castro")
/// local name = p.get_name()
/// ```
/// 
/// In Python:
/// ```python
/// p = Person("Jordan", 23)
/// // etc
/// ```
/// 
/// In JS/easyjs:
/// ```js
/// let p = new Person("Jordan", 23);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_module_add_object(module_ptr: *mut Module, name: *const c_char, object_constructor: Func, opaque: *mut c_void) {
    assert_initiated!();

    if module_ptr.is_null() || name.is_null() {
        return;
    }

    // Borrow module
    let module_borrow = unsafe{Module::from_borrow(module_ptr)};
    let name_borrow = borrow_string!(name);

    // Add
    module_borrow.add_object(name_borrow, object_constructor, opaque); 
}