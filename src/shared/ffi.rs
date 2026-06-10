// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
/// Convert a borrowed C string (const char *) into a Rust &str.
#[macro_export]
macro_rules! borrow_string {
    ($cstr:expr) => {{
        if $cstr.is_null() {
            ""
        } else {
            #[allow(unused_unsafe)]
            unsafe {
                let c_str = std::ffi::CStr::from_ptr($cstr);
                c_str.to_str().unwrap_or("")
            }
        }
    }};
}

/// Convert a owned C string (i.e. owned by us now.) into a Rust String.
///
/// The C memory will be freed automatically, and you get a nice clean String!
#[macro_export]
macro_rules! own_string {
    ($cstr:expr) => {{
        if $cstr.is_null() {
            String::new()
        } else {
            #[allow(unused_unsafe)]
            let owned_string = unsafe { std::ffi::CString::from_raw($cstr) };

            owned_string
                .into_string()
                .unwrap_or_else(|_| String::from("Invalid UTF-8"))
        }
    }};
}

/// Create a raw string from &str.
///
/// Remember to FREE THIS!
#[macro_export]
macro_rules! create_raw_string {
    ($rstr:expr) => {{ 
        std::ffi::CString::new($rstr).unwrap().into_raw() }};
}

/// Free a raw sring
#[macro_export]
    macro_rules! free_raw_string {
        ($rptr:expr) => {{
            if !$rptr.is_null() {
                let _ = std::ffi::CString::from_raw($rptr);
            }
        }};
    }


/// simple Borrow a Var.
#[macro_export]
macro_rules! borrow_var {
    ($var:expr) => {{
        unsafe{ pxs_Var::from_borrow($var) }   
    }};
}

/// Own a Var.
#[macro_export]
macro_rules! own_var {
    ($var:expr) => {{
        let v = pxs_Var::from_raw($var);
        v
    }};
}

/// Generic from_raw for ThreadLanguageState
fn generic_from_raw<T>(pointer: *mut T) {
    let _: T = unsafe { *Box::from_raw(pointer) };
}

/// Useful structure that wraps a Mut pointer of type `T`.
/// Use it for thread_local language state.
/// 
/// Wraps a pointer and drops it via Box when Drop is called.
pub(crate) struct ThreadLanguageState<T> {
    pointer: *mut T
}

impl<T> ThreadLanguageState<T> {
    /// Create a new `LanguageState` wrapper. `pointer` must be freeable via reboxing.
    pub fn new(pointer: *mut T) -> Self {
        ThreadLanguageState { pointer }
    }

    /// Get the raw pointer.
    pub fn get_ptr(&self) -> *mut T {
        self.pointer
    }
}

impl<T> Drop for ThreadLanguageState<T> {
    fn drop(&mut self) {
        if self.pointer.is_null() {
            return;
        }
        generic_from_raw::<T>(self.pointer);
        self.pointer = std::ptr::null_mut();
    }
}

unsafe impl<T> Sync for ThreadLanguageState<T> {}
unsafe impl<T> Send for ThreadLanguageState<T> {}

// /// Wraps a pointer and allows it to be passed around threads.
// /// 
// /// It does not free the pointer.
// pub(crate) struct ThreadSafePointer<T> {
//     pointer: *mut T
// }

// impl<T> ThreadSafePointer<T> {
//     pub fn new(pointer: *mut T) -> Self {
//         ThreadSafePointer { pointer }
//     }

//     pub fn get_ptr(&self) -> *mut T {
//         self.pointer
//     }
// }

// unsafe impl<T> Sync for ThreadSafePointer<T> {}
// unsafe impl<T> Send for ThreadSafePointer<T> {}