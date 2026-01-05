use super::var::Var;
use std::{collections::HashMap, ffi::c_void, sync::{Mutex, OnceLock}};

/// Function reference used in C.
/// 
/// argc: i32, The number of args.
/// argc: *const *mut Var, a C Array of args.
/// opaque: *mut c_void, opaque user data. 
/// 
/// Func handles it's own memory, so no need to free the *mut Var returned or the argvs.
/// 
/// But if you use any Vars within the function, you will have to free them before the function returns.
pub type Func = unsafe extern "C" fn(
    argc: usize, 
    argv: *mut *mut Var, 
    opaque: *mut c_void
) -> *mut Var;

/// Basic rust structure to track Funcs and opaques together.
pub struct Function {
    pub func: Func,
    pub opaque: *mut c_void
}

unsafe impl Send for Function {}
unsafe impl Sync for Function {}

/// Lookup state structure
pub struct FunctionLookup {
    /// Function hash shared between all runtimes.
    /// 
    /// Negative numbers are valid here.
    pub function_hash: HashMap<i32, Function>
}

impl FunctionLookup {
    pub fn get_function(&self, idx: i32) -> Option<&Function> {
        self.function_hash.get(&idx)
    }
    pub fn add_function(&mut self, func: Func, opaque: *mut c_void) -> i32 {
        // TODO: Allow for negative idxs.
        self.function_hash.insert(self.function_hash.len() as i32, Function { func, opaque });

        return (self.function_hash.len() - 1) as i32;
    }
}

/// The function lookup!
static FUNCTION_LOOKUP: OnceLock<Mutex<FunctionLookup>> = OnceLock::new();

/// Get the function lookup global state. Shared between all runtimes.
pub fn get_function_lookup() -> std::sync::MutexGuard<'static, FunctionLookup> {
    FUNCTION_LOOKUP.get_or_init(|| {
        Mutex::new(FunctionLookup {
            function_hash: HashMap::new(),
        })
    })
    .lock()
    .unwrap()
}