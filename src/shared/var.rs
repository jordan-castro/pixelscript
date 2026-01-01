use std::{
    ffi::{CStr, CString, c_char, c_void},
    ptr,
};

use anyhow::{Error, anyhow};

/// Macro for writing out the Var:: get methods.
macro_rules! write_func {
    ($ (($func_name:ident, $field_name:ident, $ret_type:ty, $tag_variant:path) ),* $(,)?) => {
        $(
            #[doc = concat!("Returns the ", stringify!($ret_type), " value if the tag is ", stringify!($tag_variant), ".")]
            pub fn $func_name(&self) -> Result<$ret_type, Error> {
                if self.tag == $tag_variant {
                    unsafe {
                        Ok(self.value.$field_name)
                    }
                } else {
                    Err(anyhow!("Var is not the expected type of {:#?}", $tag_variant))
                }
            }
        )*
    };
}

/// Macro for writing out the new_t methods in Var
macro_rules! write_new_methods {
    ($($t:ty, $func:ident, $vt:expr, $vn:ident);*) => {
        $(
            pub fn $func(val:$t) -> Self {
                Self {
                    tag: $vt,
                    value: VarValue { $vn: val },
                }
            }
        )*
    };
}

/// Macro for writing out the is_t methods in Var
macro_rules! write_is_methods {
    ($($func:ident, $vt:expr);*) => {
        $(
            pub fn $func(&self) -> bool {
                self.tag == $vt
            }
        )*
    };
}

// Macro for writing out the FromVars
macro_rules! implement_from_var {
    ($($t:ty, $func:ident);*) => {
        $(
            impl FromVar for $t {
                fn from_var(var:&Var) -> Result<Self, Error> {
                    var.$func()
                }
            }
        )*
    };
}

/// This represents the variable type that is being read or created.
#[repr(u32)]
#[derive(Debug, PartialEq)]
pub enum VarType {
    Int32,
    Int64,
    UInt32,
    UInt64,
    String,
    Bool,
    Float32,
    Float64,
    // Lua (nil), Python (None), JS/easyjs (null)
    Null,
    // Array,
    // // Python only
    // Dict,
    // // Lua only
    // Tree,
    // // JS/Python only
    // Object
}

/// The Variables actual value union.
#[repr(C)]
pub union VarValue {
    pub i32_val: i32,
    pub i64_val: i64,
    pub u32_val: u32,
    pub u64_val: u64,
    pub string_val: *mut c_char,
    pub bool_val: bool,
    pub f32_val: f32,
    pub f64_val: f64,
    pub null_val: *const c_void,
}

/// The Variable struct that can be accessed directly from C.
#[repr(C)]
pub struct Var {
    pub tag: VarType,
    pub value: VarValue,
}

// Rust specific functions
impl Var {
    pub fn get<T: FromVar>(&self) -> Result<T, Error> {
        T::from_var(self)
    }

    /// Get the Rust string from the Var.
    pub fn get_string(&self) -> Result<String, Error> {
        if self.tag == VarType::String {
            unsafe {
                if self.value.string_val.is_null() {
                    return Err(anyhow!("String pointer is null"));
                }

                let c_str = CStr::from_ptr(self.value.string_val);
                let res = c_str.to_str();
                if res.is_err() {
                    return Err(anyhow!(res.err().unwrap()));
                }

                Ok(res.unwrap().to_string())
            }
        } else {
            Err(anyhow!("Var is not a string."))
        }
    }

    /// Create a new String var.
    ///
    /// The memory is leaked and needs to be freed eventually. It is freed by Var::free_var(). And done so automatically
    /// by the library.
    pub fn new_string(val: String) -> Self {
        let cstr = CString::new(val).expect("Could not create CString.");

        Var {
            tag: VarType::String,
            value: VarValue {
                string_val: cstr.into_raw(),
            },
        }
    }

    /// Creates a new Null var.
    ///
    /// No need to free, or any of that. It cretes a *const c_void
    pub fn new_null() -> Self {
        Var {
            tag: VarType::Null,
            value: VarValue {
                null_val: ptr::null(),
            },
        }
    }

    /// Convert a Vec<Var> into **Var (*const *mut Var)
    ///
    /// !Important This will leak memory which MUST BE FREED.
    /// Usually handled by the library when using within Function callbacks.
    pub fn make_pointer_array(vars: Vec<Self>) -> *mut *mut Self {
        // Create a pointer array from Vec
        let mut pointer_array: Vec<*mut Self> = vars
            .into_iter()
            .map(|v| Box::into_raw(Box::new(v)))
            .collect();

        // Leak array
        let argv = pointer_array.as_mut_ptr();

        // Forget about it
        std::mem::forget(pointer_array);

        // listo
        argv
    }

    /// Free a pointer array of Vars
    pub unsafe fn free_pointer_array(argv: *mut *mut Var, argc: usize) {
        if argv.is_null() {
            return;
        }

        unsafe {
            // Recover ptr array
            let ptrs = Vec::from_raw_parts(argv, argc, argc);

            // Drop each with a Box
            for &ptr in &ptrs {
                if !ptr.is_null() {
                    let _ = Box::from_raw(ptr);
                }
            }
        }
    }

    write_func!(
        (get_i32, i32_val, i32, VarType::Int32),
        (get_u32, u32_val, u32, VarType::UInt32),
        (get_i64, i64_val, i64, VarType::Int64),
        (get_u64, u64_val, u64, VarType::UInt64),
        (get_bool, bool_val, bool, VarType::Bool),
        (get_f32, f32_val, f32, VarType::Float32),
        (get_f64, f64_val, f64, VarType::Float64)
    );

    // $t:ty, $func:ident, $vt:expr, $vn:ident
    write_new_methods! {
        i32, new_i32, VarType::Int32, i32_val;
        i64, new_i64, VarType::Int64, i64_val;
        u32, new_u32, VarType::UInt32, u32_val;
        u64, new_u64, VarType::UInt64, u64_val;
        f32, new_f32, VarType::Float32, f32_val;
        f64, new_f64, VarType::Float64, f64_val;
        bool, new_bool, VarType::Bool, bool_val
    }

    write_is_methods! {
        is_i32, VarType::Int32;
        is_i64, VarType::Int64;
        is_u32, VarType::UInt32;
        is_u64, VarType::UInt64;
        is_f32, VarType::Float32;
        is_f64, VarType::Float64;
        is_bool, VarType::Float64;
        is_string, VarType::String;
        is_null, VarType::Null
    }
}

impl Drop for Var {
    fn drop(&mut self) {
        if self.tag == VarType::String {
            unsafe {
                // Free the mem
                if !self.value.string_val.is_null() {
                    let _ = CString::from_raw(self.value.string_val);
                    self.value.string_val = ptr::null_mut();
                }
            }
        }
    }
}

/// Simple trait for Vars to get the type when writing code out.
pub trait FromVar: Sized {
    fn from_var(var: &Var) -> Result<Self, Error>;
}

implement_from_var! {
    i32, get_i32;
    u32, get_u32;
    String, get_string;
    f32, get_f32;
    f64, get_f64;
    bool, get_bool
}
