use std::ffi::{CStr, CString, c_char, c_void};
use shared::{
    var::Var,
    func::Func
};

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