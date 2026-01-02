use std::{ffi::{CStr, CString, c_char, c_void}, ptr};
use shared::{
    var::Var,
    func::Func
};

use crate::shared::{PtrMagic, module::Module};

pub mod shared;
pub mod lua;

/// Convert a borrowed C string (const char *) into a Rust &str.
macro_rules! convert_borrowed_string {
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
macro_rules! convert_owned_string {
    ($cstr:expr) => {{
        if $cstr.is_null() {
            String::new()
        } else {
            let owned_string = unsafe { CString::from_raw($cstr) };

            owned_string.into_string().unwrap_or_else(|_| String::from("Invalid UTF-8"))  
        }
    }};
}

/// Create a raw string from &str.
/// 
/// Remember to FREE THIS!
macro_rules! create_raw_string {
    ($rstr:expr) => {{
        CString::new($rstr).unwrap().into_raw()   
    }};
}

/// Add a variable to the __main__ context.
/// Gotta pass in a name, and a Variable value.
#[unsafe(no_mangle)]
pub extern "C" fn pixelmods_add_variable(name: *const c_char, variable: Var) {
    // Get string as rust.
    let r_str = convert_borrowed_string!(name);
    if r_str.is_empty() {
        return;
    }

    // Add variable to lua context
    lua::var::add_variable(r_str, variable);
}

/// Add a callback to the __main__ context.
/// Gotta pass in a name, Func, and a optionl *void opaque data type
#[unsafe(no_mangle)]
pub extern "C" fn pixelmods_add_callback(name: *const c_char, func: Func, opaque: *mut c_void) {
    // Get rust name
    let name_str = convert_borrowed_string!(name);
    if name_str.is_empty() {
        return;
    }

    // Add Function to lua context
    
}

/// Execute some lua code. Will return a String, an empty string means that the 
/// code executed succesffuly
/// 
/// The result needs to be freed by calling `pixelmods_free_str` 
#[unsafe(no_mangle)]
pub extern "C" fn pixelmods_exec_lua(code: *const c_char, file_name: *const c_char) -> *const c_char {
    // First convert code and file_name to rust strs
    let code_str = convert_borrowed_string!(code);
    if code_str.is_empty() {
        return create_raw_string!("Code is empty")
    }
    let file_name_str = convert_borrowed_string!(file_name);
    if file_name_str.is_empty() {
        return create_raw_string!("File name is empty")
    }

    // Execute and get result
    let result = lua::execute(code_str, file_name_str);

    create_raw_string!(result)
}

/// Free the string created by the pixelmods library
#[unsafe(no_mangle)]
pub extern "C" fn pixelmods_free_str(string: *mut c_char) {
    if !string.is_null() {
        unsafe {
            // Let the string go out of scope to be dropped
            let _ = CString::from_raw(string);
        }
    }
}

/// Create a new pixelmods Module.
/// 
/// Just pass in a name, PM will handle it's memory from here on out.
#[unsafe(no_mangle)]
pub extern "C" fn pixelmods_new_module(name: *mut c_char) -> *mut Module {
    if name.is_null() {
        return ptr::null_mut();
    }
    let name_string = convert_owned_string!(name);

    Module::new(name_string).into_raw()
}

/// Add a callback to a module.
/// 
/// Pass in the modules pointer and callback paramaters.
/// 
/// PM will handle the string memory from here on out.
#[unsafe(no_mangle)]
pub extern "C" fn pixelmods_module_add_callback(module_ptr: *mut Module, name: *mut c_char, func: Func, opaque: *mut c_void) {
    if module_ptr.is_null() {
        return;
    }

    if name.is_null() {
        return;
    }

    // Get actual data
    let module = unsafe {Module::from_borrow(module_ptr)};
    let name_owned = convert_owned_string!(name);

    // Now add callback
    module.add_callback(name_owned.as_str(), func, opaque);
}

/// Add a Varible to a module.
/// 
/// Pass in the module pointer and variable params.
/// 
/// PM will handle the string memory from here on out.
#[unsafe(no_mangle)]
pub extern "C" fn pixelmods_module_add_variable(module_ptr: *mut Module, name: *mut c_char, variable: Var) {
    if module_ptr.is_null() {
        return;
    }

    if name.is_null() {
        return;
    }

    let module = unsafe {Module::from_borrow(module_ptr)};
    let name_owned = convert_owned_string!(name);

    // Now add variable
    module.add_variable(&name_owned, variable);
}

/// Add the module finally to the runtime.
/// 
/// After this you can forget about the ptr since PM handles it.
#[unsafe(no_mangle)]
pub extern "C" fn pixelmods_add_module(module_ptr: *mut Module) {
    if module_ptr.is_null() {
        return;
    }

    let module = Module::from_raw(module_ptr);

    // LUA
    lua::module::add_module(module);

    // Module gets dropped here, and that is good!
}