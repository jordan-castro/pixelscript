use std::{ffi::c_void, sync::Arc};

pub mod func;
pub mod module;
pub mod object;
pub mod var;

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

    /// Add a global variable to the runtime.
    fn add_variable(name: &str, variable: &var::Var);
    /// Add a object variable to the runtime.
    fn add_object_variable(name: &str, idx: i32);
    /// Add a global callback to the runtime.
    fn add_callback(name: &str, idx: i32);
    /// Add a global module to the runtime.
    fn add_module(source: Arc<module::Module>);
    /// Add a Object constructor globally to the runtime.
    fn add_object(name: &str, callback: func::Func, opaque: *mut c_void);
    /// Execute a script in this runtime.
    fn execute(code: &str, file_name: &str) -> String;
}

/// Public enum for supported runtimes.
#[repr(C)]
pub enum PixelScriptRuntime {
    Lua,
    Python,
    JavaScript,
    Easyjs
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