// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{
    ffi::{CString, c_char, c_void}, sync::{Arc, LazyLock}
};

use etffi::{ptr_magic::{PtrMagic, ThreadSafePointer}, cstring::CStringSafe};

use crate::{
    own_var, shared::{var::{pxs_Var, pxs_VarT}}
};

/// Helper methods/macros for using PixelScript
pub mod ffi;
/// The internal PixelScript function logic.
pub mod func;
/// The internal PixelScript Module structure.
pub mod module;
/// The internal PixelScript PixelObject logic.
pub mod object;
pub mod utils;
/// The internal PixelScript Var logic.
pub mod var;
pub mod arena;

/// cbindgen:ignore
/// This is a internal function used in `pxs_utils.h` to allow bridge code to work with rust strings.
#[unsafe(no_mangle)]
unsafe extern "C" fn pxsutils_freestring(ptr: *mut core::ffi::c_char) {
    if !ptr.is_null() {
        let _ = etffi::own_string!(ptr);
    }
}

#[allow(non_camel_case_types)]
/// Function Type for Loading a file.
pub type pxs_LoadFileFn = unsafe extern "C" fn(file_path: *const c_char) -> pxs_VarT;

#[allow(non_camel_case_types)]
/// Function Type for reading a Dir. Should return a `pxs_List`
pub type pxs_ReadDirFn = unsafe extern "C" fn(dir_path: *const c_char) -> pxs_VarT;

#[allow(non_camel_case_types)]
pub type pxs_Opaque = *mut c_void;

/// Error type in PXS
pub(crate) type PxsError = String;

/// Generic Result type in PXS
pub(crate) type PxsRes<T> = Result<T, PxsError>;

/// Main Result type in PXS
pub(crate) type PxsResult = PxsRes<pxs_Var>;

/// This is the PixelScript state.
pub(crate) struct PixelState {
    pub load_file: Option<pxs_LoadFileFn>,
    pub read_dir: Option<pxs_ReadDirFn>,
}

impl PtrMagic for PixelState {}

/// The State static variable for PixelScript.
static PIXEL_STATE: LazyLock<ThreadSafePointer<PixelState>> = LazyLock::new(|| {
    ThreadSafePointer::<PixelState>::new_owned(init_state())
});

/// This is a internal macro to create a PxsError type.
#[macro_export]
macro_rules! pxs_error {
    ($($arg:tt)*) => {
        Err(format!("{}", format_args!($($arg)*)))
    }
}

fn init_state() -> *mut PixelState {
    PixelState{
        load_file: None,
        read_dir: None
    }.into_raw()
}

/// Set `read_file` function in PixelState global.
pub(crate) fn set_read_file(func: pxs_LoadFileFn) {
    unsafe { 
        (*PIXEL_STATE.get_ptr()).load_file = Some(func);
    }
}

/// Set `read_dir` function in PixelState global
pub(crate) fn set_read_dir(func: pxs_ReadDirFn) {
    unsafe {
        (*PIXEL_STATE.get_ptr()).read_dir = Some(func);
    }
}

/// Read a file using pxs api.
/// This must be set by host language.
pub fn read_file(file_path: &str) -> String {
    // Get callback
    let cbk = unsafe { (*PIXEL_STATE.get_ptr()).load_file };
    if cbk.is_none() {
        return String::new();
    }
    let cbk = cbk.unwrap();

    let mut cstring = CStringSafe::new();
    let res = unsafe { cbk(cstring.new_string(file_path)) };
    if res.is_null() {
        return String::new();
    }

    let var = own_var!(res);
    if !var.is_string() {
        return String::new();
    }

    var.get_string().unwrap()
}

/// Read a Directory using pxs api.
/// This must be set by host language.
pub fn read_file_dir(dir_path: &str) -> Vec<String> {
    let cbk = unsafe { (*PIXEL_STATE.get_ptr()).read_dir };
    if cbk.is_none() {
        return vec![];
    }
    let cbk = cbk.unwrap();

    // Convert to c_str
    let c_str = CString::new(dir_path).unwrap();
    let dir_path_cstr = c_str.as_ptr();
    let res = unsafe { cbk(dir_path_cstr) };

    // Check null
    if res.is_null() {
        return vec![];
    }

    // Own variable! The memory will drop at the end dayo!
    let var = own_var!(res);
    if !var.is_list() {
        return vec![];
    }

    // remove arena reference
    // var.remove_arena_re();

    // Get all strings!
    var.get_list()
        .unwrap()
        .vars
        .iter()
        .map(|v| v.get_string().unwrap_or(String::new()).clone())
        .collect()
}

/// The trait to use for PixelScrpipting
pub trait PixelScript {
    /// Start the runtime.
    fn start();

    /// Stop the runtime.
    fn stop();

    /// Add a global module to the runtime.
    fn add_module(source: Arc<module::pxs_Module>);

    /// Execute a script in this runtime.
    fn execute(code: &str, file_name: &str) -> PxsResult;

    /// Evaluate a script in this runtime. Returns a pxs_Var.
    fn eval(code: &str, name: &str) -> PxsResult;

    /// Some langauges (pocketpy) need to be explicitly told that a new thread is starting.
    /// For most languages this is NOT needed.
    fn start_thread();

    /// Some languages (pocketpy) need to be expliclity told that a recent thread has stopped.
    /// For most languages this is NOT needed.
    fn stop_thread();

    /// Clear the current threads state.
    fn clear();

    /// Compile and save for future use.
    /// Pass in a optional global scope, if null, defaults to empty Map.
    /// Result will be a list with: [Runtime, Compiled Object, ...]
    fn compile(code: &str, global_scope: pxs_Var) -> PxsResult;

    /// Execute a code object.
    /// The code variable will always be a List with: [Runtime, Compiled Object, ...].
    /// Pass in optional local scope that will be included along with the compiled scope.
    fn exec_object(code: pxs_Var, local_scope: pxs_Var) -> PxsResult;

    /// For debugging purposes. Return a string which explains the current state.
    fn debug() -> String;

    /// Call the garbage collector. Will also free internal types.
    fn garbage_collect();
}

/// Public enum for supported runtimes.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub enum pxs_Runtime {
    /// Lua v5.4 with mlua.
    pxs_Lua = 0,
    /// Python v3.x with pocketpy.
    pxs_Python = 1,
    /// ES 2020 using rquickjs
    pxs_JavaScript = 2,
    pxs_Wren = 3
}

impl pxs_Runtime {
    pub fn into_i64(&self) -> i64 {
        match self {
            pxs_Runtime::pxs_Lua => 0,
            pxs_Runtime::pxs_Python => 1,
            pxs_Runtime::pxs_JavaScript => 2,
            pxs_Runtime::pxs_Wren => 3
        }
    }

    pub fn from_i64(val: i64) -> Option<Self> {
        match val {
            0 => Some(Self::pxs_Lua),
            1 => Some(Self::pxs_Python),
            2 => Some(Self::pxs_JavaScript),
            3 => Some(Self::pxs_Wren),
            _ => None,
        }
    }

    /// Gets the runtime from a pxs_Var pointer. The pointer is borrowed.
    pub unsafe fn from_var_ptr(var: *mut pxs_Var) -> Option<Self> {
        if var.is_null() {
            return None;
        }
        let borrow = unsafe { pxs_Var::from_borrow(var) };
        let int_val = borrow.get_i64();
        if let Ok(int_val) = int_val {
            Self::from_i64(int_val)
        } else {
            None
        }
    }

    /// Gets the runtime from a pxs_Var
    pub fn from_var(var: &pxs_Var) -> Option<Self> {
        let int_val = var.get_i64();
        if let Ok(int_val) = int_val {
            Self::from_i64(int_val)
        } else {
            None
        }
    }

    /// Turns current runtime into a `pxs_Int64`
    pub fn into_var(&self) -> pxs_Var {
        let idx = self.into_i64();
        pxs_Var::new_i64(idx)
    }
}

/// PXS PTR name string
pub const PXS_PTR_NAME: &str = "_pxs_ptr";

/// PXS __pxs__ internal method
pub const PXS_METHOD_NAME: &str = "__pxs__";