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
    ffi::{CStr, CString, c_char},
    sync::{Arc, OnceLock},
};

use parking_lot::{ReentrantMutex, ReentrantMutexGuard};

use crate::own_string;

/// Helper methods/macros for using PixelScript
pub mod ffi;
/// The internal PixelScript function logic.
pub mod func;
/// The internal PixelScript Module structure.
pub mod module;
/// The internal PixelScript PixelObject logic.
pub mod object;
/// The internal PixelScript Var logic.
pub mod var;

/// Type for DirHandle.
///
/// Host owns memory.
#[repr(C)]
pub struct DirHandle {
    /// The Length of the array
    pub length: usize,
    /// The array values
    pub values: *mut *mut c_char,
}

impl DirHandle {
    /// Empty Dir Handle
    pub fn empty() -> Self {
        DirHandle { length: 0, values: std::ptr::null_mut() }
    }

    /// Get a Vec<String> from the DirHandle.
    pub fn into_vec(&self) -> Vec<String> {
        if self.length == 0 || self.values.is_null() {
            return vec![];
        }
        let args = unsafe { std::slice::from_raw_parts_mut(self.values, self.length) };
        let mut res = vec![];
        for &ptr in args.iter() {
            if ptr.is_null() {
                continue;
            }

            let c_str = unsafe { CStr::from_ptr(ptr) };
            let str_slice = c_str.to_str().unwrap_or("<Invalid UTF-8>");
            res.push(str_slice.to_string().clone());
        }
        res
    }
}

impl Drop for DirHandle {
    fn drop(&mut self) {
        if self.length == 0 || self.values.is_null() {
            return;
        }

        // Go through each and free them
        let args = unsafe { std::slice::from_raw_parts_mut(self.values, self.length) };

        for arg in args.iter() {
            let _ = own_string!(arg.to_owned());
        }
    }
}

/// Function Type for Loading a file.
pub type LoadFileFn = unsafe extern "C" fn(file_path: *const c_char) -> *mut c_char;
/// Function Type for writing a file.
pub type WriteFileFn = unsafe extern "C" fn(file_path: *const c_char, contents: *const c_char);
/// Function Type for reading a Dir.
pub type ReadDirFn = unsafe extern "C" fn(dir_path: *const c_char) -> DirHandle;

/// This is the PixelScript state.
pub(crate) struct PixelState {
    pub load_file: RefCell<Option<LoadFileFn>>,
    pub write_file: RefCell<Option<WriteFileFn>>,
    pub read_dir: RefCell<Option<ReadDirFn>>,
}

/// The State static variable for Lua.
static PIXEL_STATE: OnceLock<ReentrantMutex<PixelState>> = OnceLock::new();

/// Get the state of LUA.
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

/// Read a file
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

/// Write a file
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

/// Read a Directory.
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
    res.into_vec()
}

/// A shared trait for converting from/to a pointer. Specifically a (* mut Self)
pub trait PtrMagic: Sized {
    /// Moves the object to the heap and returns a raw pointer.
    /// Caller owns this memory but don't worry about freeing it. The library frees it somewhere.
    fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    /// Safety: Only call this on a pointer created via `into_raw`.
    fn from_raw(ptr: *mut Self) -> Self {
        unsafe { *Box::from_raw(ptr) }
    }

    /// Build from a Ptr but only get a reference, this means that the caller will still own the memory
    unsafe fn from_borrow<'a>(ptr: *mut Self) -> &'a mut Self {
        unsafe {
            assert!(!ptr.is_null(), "Attempted to borrow a null pointer.");
            &mut *ptr
        }
    }
}

/// The trait to use for PixelScrpipting
pub trait PixelScript {
    /// Start the runtime.
    fn start();
    /// Stop the runtime.
    fn stop();

    /// Add a global module to the runtime.
    fn add_module(source: Arc<module::Module>);
    /// Execute a script in this runtime.
    fn execute(code: &str, file_name: &str) -> String;
    /// Allows the language to start a new thread. In this new thread all callbacks/objects/variables will be empty.
    fn start_thread();
    /// Tells the language that we just finished the most recent started thread.
    fn stop_thread();
}

/// Public enum for supported runtimes.
#[repr(C)]
pub enum PixelScriptRuntime {
    Lua,
    Python,
    JavaScript,
    Easyjs,
}

impl PixelScriptRuntime {
    pub fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Lua),
            1 => Some(Self::Python),
            2 => Some(Self::JavaScript),
            3 => Some(Self::Easyjs),
            _ => None, // Handle invalid integers from C safely
        }
    }
}
