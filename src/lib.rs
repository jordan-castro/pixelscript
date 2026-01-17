// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// use rand::{Rng, SeedableRng, distr::Alphanumeric, rngs::SmallRng};
use shared::{func::Func, var::Var};
use std::{
    ffi::{CString, c_char, c_void},
    ptr,
    sync::Arc,
};

#[cfg(feature = "lua")]
use crate::lua::LuaScripting;
#[cfg(feature = "python")]
use crate::python::PythonScripting;

use crate::shared::{
    LoadFileFn, PixelScript, PixelScriptRuntime, PtrMagic, ReadDirFn, WriteFileFn, func::{clear_function_lookup, lookup_add_function}, get_pixel_state, module::Module, object::{FreeMethod, PixelObject, clear_object_lookup, lookup_add_object}, var::{ObjectMethods, VarType}
};

pub mod shared;

#[cfg(feature = "include-core")]
pub mod core;
#[cfg(feature = "lua")]
pub mod lua;
#[cfg(feature = "python")]
pub mod python;

/// Macro to wrap features
macro_rules! with_feature {
    ($feature:expr, $logic:block) => {
        #[cfg(feature=$feature)]
        {
            $logic
        }
    };
    ($feature:literal, $logic:block, $fallback:block) => {{
        #[cfg(feature = $feature)]
        {
            $logic
        }
        #[cfg(not(feature = $feature))]
        {
            $fallback
        }
    }};
}

/// Assert that the module is initiated.
macro_rules! assert_initiated {
    () => {{
        unsafe {
            assert!(IS_INIT, "Pixel script library is not initialized.");
            // if !IS_INIT {
            // panic!("Pixel Script library is not initialized.");
            // }
        }
    }};
}

// /// Add the methods for creating a pixel var.
// macro_rules! make_pixel_var {
//     ($($ffi_name:ident, $internal_method:ident, $t:ty);*) => {
//         $(
//         #[unsafe(no_mangle)]
//         pub extern "C" fn $ffi_name(val: $t) -> Var {
//             Var::$internal_method(val)
//         })*
//     };
// }

// /// Create a random string.
// ///
// /// Used for PixelTypes
// fn random_string() -> String {
//     const STRING_LEN: usize = 8;
//     let mut rng = SmallRng::from_rng(&mut rand::rng());

//     (0..STRING_LEN)
//         .map(|_| rng.sample(Alphanumeric) as char)
//         .collect()
// }

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

            with_feature!("python", {
                PythonScripting::start();
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
    clear_function_lookup();
    // get_function_lookup().function_hash.clear();
    // Drop object lookup
    clear_object_lookup();
    // get_object_lookup().object_hash.clear();

    with_feature!("lua", {
        LuaScripting::stop();
    });

    with_feature!("python", {
        PythonScripting::stop();
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
) -> *mut c_char {
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
    let result = LuaScripting::execute(code_str, file_name_str);

    create_raw_string!(result)
}

/// Execute some Python code. Will return a String, an empty string means that the code executed successfully.
///
/// The result needs to be freed by calling `pixelscript_free_str`
#[unsafe(no_mangle)]
#[cfg(feature = "python")]
pub extern "C" fn pixelscript_exec_python(
    code: *const c_char,
    file_name: *const c_char,
) -> *mut c_char {
    assert_initiated!();

    // Borrow code and name
    let code_borrow = borrow_string!(code);
    if code_borrow.is_empty() {
        return create_raw_string!("Code is empty");
    }
    let file_name_borrow = borrow_string!(file_name);
    if file_name_borrow.is_empty() {
        return create_raw_string!("File name is empty");
    }

    // Execute
    let result = PythonScripting::execute(code_borrow, file_name_borrow);

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
pub extern "C" fn pixelscript_add_callback(
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

    // Mangle the name
    let full_name = format!("_{}{}", module.name, name_str);

    // Save the callback
    let idx = lookup_add_function(&full_name, func, opaque);

    // Now add callback
    module.add_callback(name_str, &full_name, idx);
}

/// Add a Varible to a module.
///
/// Pass in the module pointer and variable params.
///
/// Variable ownership is transfered.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_variable(
    module_ptr: *mut Module,
    name: *const c_char,
    variable: *mut Var,
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
pub extern "C" fn pixelscript_add_submodule(parent_ptr: *mut Module, child_ptr: *mut Module) {
    assert_initiated!();
    if parent_ptr.is_null() || child_ptr.is_null() {
        return;
    }

    let parent = unsafe { Module::from_borrow(parent_ptr) };
    // Own child
    let child = Module::from_raw(child_ptr);

    parent.add_module(child.clone());

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
    with_feature!("python", {
        PythonScripting::add_module(Arc::clone(&module));
    });

    // Module gets dropped here, and that is good!
}

/// Optionally free a module if you changed your mind.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_free_module(module_ptr: *mut Module) {
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
pub extern "C" fn pixelscript_new_object(
    ptr: *mut c_void,
    free_method: FreeMethod,
    type_name: *const c_char,
) -> *mut PixelObject {
    assert_initiated!();
    if ptr.is_null() || type_name.is_null() {
        return ptr::null_mut();
    }

    // Borrow type_name
    let type_name = borrow_string!(type_name);

    PixelObject::new(ptr, free_method, type_name).into_raw()
}

/// Add a callback to a object.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_object_add_callback(
    object_ptr: *mut PixelObject,
    name: *const c_char,
    callback: Func,
    opaque: *mut c_void,
) {
    assert_initiated!();

    if object_ptr.is_null() || name.is_null() {
        return;
    }

    // Borrow ptr
    let object_borrow = unsafe { PixelObject::from_borrow(object_ptr) };
    let name_borrow = borrow_string!(name);

    // Add to function lookup
    let full_name = format!("_{}{}", object_borrow.type_name, name_borrow);
    let idx = lookup_add_function(full_name.as_str(), callback, opaque);

    object_borrow.add_callback(name_borrow, full_name.as_str(), idx);
}

/// Add a object to a Module.
///
/// This essentially makes it so that when constructing this Module, this object is instanced.
///
/// Depending on the language, you may need to wrap the construction. For example lua:
/// ```lua
/// -- Let's say we have a object "Person"
/// local p = Person("Jordan", 23)
/// p:set_name("Jordan Castro")
/// local name = p:get_name()
///
/// -- Although you could also do
/// local p = Person("Jordan", 23)
/// p.set_name(p, "Jordan") -- You get the idea
/// ```
///
/// In Python:
/// ```python
/// p = Person("Jordan", 23)
/// # etc
/// ```
///
/// In JS/easyjs:
/// ```js
/// let p = new Person("Jordan", 23);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_object(
    module_ptr: *mut Module,
    name: *const c_char,
    object_constructor: Func,
    opaque: *mut c_void,
) {
    // Save module to object
    pixelscript_add_callback(module_ptr, name, object_constructor, opaque);
}

/// Make a new Var string.
///
/// Does take ownership
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_newstring(str: *mut c_char) -> *mut Var {
    let val = own_string!(str);
    Var::new_string(val).into_raw()
}

/// Make a new Null var.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_newnull() -> *mut Var {
    Var::new_null().into_raw()
}

/// Make a new HostObject var.
///
/// If not a valid pointer, will return null
///
/// Transfers ownership
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_newhost_object(pixel_object: *mut PixelObject) -> *mut Var {
    assert_initiated!();

    if pixel_object.is_null() {
        return Var::new_null().into_raw();
    }

    // Own the pixel_object
    let pixel_owned = PixelObject::from_raw(pixel_object);
    // Arc it
    let pixel_arc = Arc::new(pixel_owned);

    // Create it in the system
    let idx = lookup_add_object(Arc::clone(&pixel_arc));

    Var::new_host_object(idx).into_raw()
}

/// Create a new variable i32.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_newi32(val: i32) -> *mut Var {
    Var::new_i32(val).into_raw()
}
/// Create a new variable u32.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_newu32(val: u32) -> *mut Var {
    Var::new_u32(val).into_raw()
}
/// Create a new variable i64.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_newi64(val: i64) -> *mut Var {
    Var::new_i64(val).into_raw()
}
/// Create a new variable u64.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_newu64(val: u64) -> *mut Var {
    Var::new_u64(val).into_raw()
}
/// Create a new variable bool.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_newbool(val: bool) -> *mut Var {
    Var::new_bool(val).into_raw()
}
/// Create a new variable f32.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_newf32(val: f32) -> *mut Var {
    Var::new_f32(val).into_raw()
}

/// Create a new variable f64
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_newf64(val: f64) -> *mut Var {
    Var::new_f64(val).into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_object_call_rt(
    runtime: PixelScriptRuntime,
    var: *mut Var,
    method: *const c_char,
    argc: usize,
    argv: *mut *mut Var,
) -> *mut Var {
    pixelscript_object_call(
        Var::new_i64(runtime as i64).into_raw(),
        var,
        method,
        argc,
        argv,
    )
}

/// Object call.
///
/// All memory is borrowed.
///
/// You can get the runtime from the first Var in any callback.
///
/// Example
/// ```C
///     // Inside a Var* method
///     Var* obj = argv[1];
///     Var name = pixelscript_object_call()
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_object_call(
    runtime: *mut Var,
    var: *mut Var,
    method: *const c_char,
    argc: usize,
    argv: *mut *mut Var,
) -> *mut Var {
    assert_initiated!();

    if var.is_null() || method.is_null() || argv.is_null() || runtime.is_null() {
        return Var::new_null().into_raw();
    }

    // Borrow runtime, var, and method, and argv
    let runtime_borrow = unsafe { Var::from_borrow(runtime) };
    let var_borrow = unsafe { Var::from_borrow(var) };
    let method_borrow = borrow_string!(method);
    let argv_borrow: &[*mut Var] = unsafe { Var::slice_raw(argv, argc) };
    let args: Vec<Var> = argv_borrow
        .iter()
        .filter(|ptr| !ptr.is_null()) // Always check for nulls from C
        .map(|&ptr| (unsafe { (*ptr).clone() }).clone()) // Dereference and clone
        .collect();

    // Check that runtime is acually a int
    let runtime = runtime_borrow.get_i64();
    if runtime.is_err() {
        return Var::new_null().into_raw();
    }

    let runtime = PixelScriptRuntime::from_i32(runtime.unwrap() as i32);
    if runtime.is_none() {
        return Var::new_null().into_raw();
    }
    let runtime = runtime.unwrap();

    // Ensure type
    let tags = vec![
        VarType::Object,
        VarType::HostObject,
        VarType::Int32,
        VarType::Int64,
        VarType::UInt64,
        VarType::UInt32,
    ];
    if !tags.contains(&var_borrow.tag) {
        return Var::new_null().into_raw();
    }

    // This is tricky since we need to know what runtime we are using...
    let var: Result<Var, anyhow::Error> = match runtime {
        PixelScriptRuntime::Lua => {
            with_feature!(
                "lua",
                { LuaScripting::object_call(var_borrow, method_borrow, &args) },
                { Ok(Var::new_null()) }
            )
        }
        PixelScriptRuntime::Python => {
            with_feature!(
                "python",
                { PythonScripting::object_call(var_borrow, method_borrow, &args) },
                { Ok(Var::new_null()) }
            )
        }
        PixelScriptRuntime::JavaScript => todo!(),
        PixelScriptRuntime::Easyjs => todo!(),
    };

    if let Ok(var) = var {
        var.into_raw()
    } else {
        Var::new_null().into_raw()
    }
}

/// Get a I32 from a var.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_get_i32(var: *mut Var) -> i32 {
    if var.is_null() {
        return -1;
    }

    let var = unsafe { Var::from_borrow(var) };

    var.get_int()
}

/// Get a I64 from a var.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_get_i64(var: *mut Var) -> i64 {
    if var.is_null() {
        return -1;
    }

    unsafe { Var::from_borrow(var).get_bigint() }
}

/// Get a U32 from a var.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_get_u32(var: *mut Var) -> u32 {
    if var.is_null() {
        return 0;
    }

    unsafe { Var::from_borrow(var).get_uint() }
}

/// Get a U64
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_get_u64(var: *mut Var) -> u64 {
    if var.is_null() {
        return 0;
    }

    unsafe { Var::from_borrow(var).get_biguint() }
}

/// Get a F32
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_get_f32(var: *mut Var) -> f32 {
    if var.is_null() {
        return -1.0;
    }

    unsafe { Var::from_borrow(var).get_float() }
}

/// Get a F64
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_get_f64(var: *mut Var) -> f64 {
    if var.is_null() {
        return -1.0;
    }

    unsafe { Var::from_borrow(var).get_bigfloat() }
}

/// Get a Bool
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_get_bool(var: *mut Var) -> bool {
    if var.is_null() {
        return false;
    }

    unsafe { Var::from_borrow(var).get_bool().unwrap() }
}

/// Get a String
///
/// DANGEROUS
///
/// You have to free this memory by calling `pixelscript_free_str`
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_get_string(var: *mut Var) -> *mut c_char {
    if var.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let string = Var::from_borrow(var).get_string().unwrap();
        create_raw_string!(string.clone())
    }
}

/// Get the pointer of the Host Object
///
/// This is "potentially" dangerous.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_get_host_object(var: *mut Var) -> *mut c_void {
    if var.is_null() {
        return ptr::null_mut();
    }

    unsafe { Var::from_borrow(var).get_host_ptr() }
}

/// Get the IDX of the PixelObject
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_get_object_idx(var: *mut Var) -> i32 {
    if var.is_null() {
        return -1;
    }

    unsafe { Var::from_borrow(var).get_object_ptr() }
}

/// Check if a variable is of a type.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_var_is(var: *mut Var, var_type: VarType) -> bool {
    if var.is_null() {
        return false;
    }

    let var_borrow = unsafe { Var::from_borrow(var) };

    var_borrow.tag == var_type
}

/// Set a function for reading a file.
///
/// This is used to load files via import, require, etc
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_set_file_reader(func: LoadFileFn) {
    assert_initiated!();
    let state = get_pixel_state();
    let mut load_file = state.load_file.borrow_mut();
    *load_file = Some(func);
}

/// Set a function for writing a file.
/// 
/// This is used to write files via pxs_json
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_set_file_writer(func: WriteFileFn) {
    assert_initiated!();
    let state = get_pixel_state();
    let mut write_file = state.write_file.borrow_mut();
    *write_file = Some(func);
}

/// Set a function for reading a directory.
///
/// This is used to read a dir.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_set_dir_reader(func: ReadDirFn) {
    assert_initiated!();
    let state = get_pixel_state();
    let mut read_dir = state.read_dir.borrow_mut();
    *read_dir = Some(func);
}

/// Free a PixelScript var.
///
/// You should only free results from `pixelscript_object_call`
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_free_var(var: *mut Var) {
    assert_initiated!();

    if var.is_null() {
        return;
    }

    let _ = Var::from_raw(var);
}

/// Tells PixelScript that we are in a new thread.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_start_thread() {
    with_feature!("lua", {
        LuaScripting::start_thread();
    });
    with_feature!("python", {
        PythonScripting::start_thread();
    });
}

/// Tells PixelScript that we just stopped the most recent thread.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_stop_thread() {
    with_feature!("lua", {
        LuaScripting::stop_thread();
    });
    with_feature!("python", {
        PythonScripting::stop_thread();
    });
}
