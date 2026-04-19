// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{
    cell::RefCell,
    ffi::{CString, c_char, c_void},
    sync::{Arc, OnceLock},
};

use anyhow::Result;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};

use crate::{
    own_string, own_var, shared::var::{pxs_Var, pxs_VarT}
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

#[allow(non_camel_case_types)]
/// Function Type for Loading a file.
pub type pxs_LoadFileFn = unsafe extern "C" fn(file_path: *const c_char) -> *mut c_char;

#[allow(non_camel_case_types)]
/// Function Type for writing a file.
pub type pxs_WriteFileFn = unsafe extern "C" fn(file_path: *const c_char, contents: *const c_char);

#[allow(non_camel_case_types)]
/// Function Type for reading a Dir. Should return a `pxs_List`
pub type pxs_ReadDirFn = unsafe extern "C" fn(dir_path: *const c_char) -> pxs_VarT;

#[allow(non_camel_case_types)]
pub type pxs_Opaque = *mut c_void;

/// This is the PixelScript state.
pub(crate) struct PixelState {
    pub load_file: RefCell<Option<pxs_LoadFileFn>>,
    pub write_file: RefCell<Option<pxs_WriteFileFn>>,
    pub read_dir: RefCell<Option<pxs_ReadDirFn>>,
}

/// The State static variable for PixelScript.
static PIXEL_STATE: OnceLock<ReentrantMutex<PixelState>> = OnceLock::new();

/// Get the state of PixelScript.
pub(crate) fn get_pixel_state() -> ReentrantMutexGuard<'static, PixelState> {
    let mutex = PIXEL_STATE.get_or_init(|| {
        ReentrantMutex::new(PixelState {
            load_file: RefCell::new(None),
            write_file: RefCell::new(None),
            read_dir: RefCell::new(None),
        })
    });
    // This will
    mutex.lock()
}

/// Read a file using pxs api.
/// This must be set by host anguage.
pub fn read_file(file_path: &str) -> String {
    // Get state
    let state = get_pixel_state();
    // Get callback
    let cbk = state.load_file.borrow();
    if cbk.is_none() {
        return String::new();
    }
    let cbk = cbk.unwrap();

    // convert to *const c_char
    let c_str = CString::new(file_path).unwrap();
    let file_path_cstr = c_str.as_ptr();
    // Call it
    let res = unsafe { cbk(file_path_cstr) };
    // Convet *mut c_char into String
    let res_owned = own_string!(res);
    res_owned
}

/// Write a file using pxs api.
/// This must be set by host language.
pub fn write_file(file_path: &str, contents: &str) {
    // Get state
    let state = get_pixel_state();
    // Get callback
    let cbk = state.write_file.borrow();
    if cbk.is_none() {
        return;
    }
    let cbk = cbk.unwrap();
    // Convert to *const c_char
    let c_file_path = CString::new(file_path).unwrap();
    let c_contents = CString::new(contents).unwrap();

    // Call it
    unsafe { cbk(c_file_path.as_ptr(), c_contents.as_ptr()) };
}

/// Read a Directory using pxs api.
/// This must be set by host language.
pub fn read_file_dir(dir_path: &str) -> Vec<String> {
    let state = get_pixel_state();
    let cbk = state.read_dir.borrow();
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

    // Get all strings!
    var.get_list()
        .unwrap()
        .vars
        .iter()
        .map(|v| v.get_string().unwrap_or(String::new()).clone())
        .collect()
}

/// A shared trait for converting from/to a pointer. Specifically a (* mut Self)
pub trait PtrMagic: Sized {
    /// Moves the object to the heap and returns a raw pointer.
    /// Caller owns this memory but don't worry about freeing it. The library frees it somewhere.
    fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    /// Get a direct *mut c_void
    fn into_void(self) -> *mut c_void {
        self.into_raw() as *mut c_void
    }

    /// Safety: Only call this on a pointer created via `into_raw`.
    fn from_raw(ptr: *mut Self) -> Self {
        assert!(!ptr.is_null(), "Attempted to own a null pointer.");
        unsafe { *Box::from_raw(ptr) }
    }

    /// Build from a Ptr but only get a reference, this means that the caller will still own the memory
    unsafe fn from_borrow<'a>(ptr: *mut Self) -> &'a mut Self {
        unsafe {
            assert!(!ptr.is_null(), "Attempted to borrow a null pointer.");
            &mut *ptr
        }
    }

    /// Completely unsafe and should only be used when cerrtain that type can be cast to Self
    unsafe fn from_borrow_void<'a>(ptr: *mut c_void) -> &'a mut Self {
        unsafe { Self::from_borrow(ptr as *mut Self) }
    }
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
    fn execute(code: &str, file_name: &str) -> Result<pxs_Var>;
    /// Evaluate a script in this runtime. Returns a pxs_Var.
    fn eval(code: &str) -> Result<pxs_Var>;
    /// Some langauges (pocketpy) need to be explicitly told that a new thread is starting.
    /// For most languages this is NOT needed.
    fn start_thread();
    /// Some languages (pocketpy) need to be expliclity told that a recent thread has stopped.
    /// For most languages this is NOT needed.
    fn stop_thread();
    /// Clear the current threads state. Optionally calls garbage collector.
    fn clear_state(call_gc: bool);
    /// Compile and save for future use.
    /// Pass in a optional global scope, if null, defaults to empty Map.
    /// Result will be a list with: [Runtime, Compiled Object, ...]
    fn compile(code: &str, global_scope: pxs_Var) -> Result<pxs_Var>;
    /// Execute a code object.
    /// The code variable will always be a List with: [Runtime, Compiled Object, ...].
    /// Pass in optional local scope that will be included along with the compiled scope.
    fn exec_object(code: pxs_Var, local_scope: pxs_Var) -> Result<pxs_Var>;
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
}

impl pxs_Runtime {
    pub fn into_i64(&self) -> i64 {
        match self {
            pxs_Runtime::pxs_Lua => 0,
            pxs_Runtime::pxs_Python => 1,
            pxs_Runtime::pxs_JavaScript => 2,
        }
    }

    pub fn from_i64(val: i64) -> Option<Self> {
        match val {
            0 => Some(Self::pxs_Lua),
            1 => Some(Self::pxs_Python),
            2 => Some(Self::pxs_JavaScript),
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
