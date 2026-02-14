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
        pxs_Var::from_raw($var)    
    }};
}