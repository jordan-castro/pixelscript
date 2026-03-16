// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{
    cell::Cell,
    ffi::{CStr, CString, c_char, c_void},
    ptr,
};

use anyhow::{Error, anyhow};

use crate::{
    borrow_string, create_raw_string, pxs_debug, shared::{
        PtrMagic,
        object::get_object, pxs_Runtime,
    }
};

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
                    Err(anyhow!("Var is not the expected type of {:#?}. It is instead a: {:#?}", $tag_variant, self.tag))
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
                    value: pxs_VarValue { $vn: val },
                    deleter: Cell::new(default_deleter)
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

// // Macro for writing out the FromVars
// macro_rules! implement_from_var {
//     ($($t:ty, $func:ident);*) => {
//         $(
//             impl FromVar for $t {
//                 fn from_var(var:&Var) -> Result<Self, Error> {
//                     var.$func()
//                 }
//             }
//         )*
//     };
// }

/// This represents the variable type that is being read or created.
#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
#[allow(non_camel_case_types)]
pub enum pxs_VarType {
    pxs_Int64,
    pxs_UInt64,
    pxs_String,
    pxs_Bool,
    pxs_Float64,
    /// Lua (nil), Python (None), JS/easyjs (null/undefined)
    pxs_Null,
    /// Lua (Tree), Python (Class), JS/easyjs (Prototype)
    pxs_Object,
    /// Host object converted when created.
    /// Lua (Tree), Python (object), JS/easyjs (Prototype think '{}')
    pxs_HostObject,
    /// Lua (Tree), Python (list), JS/easyjs (Array)
    pxs_List,
    /// Lua (Value), Python (def or lambda), JS/easyjs (anon function)
    pxs_Function,
    /// Internal object only. It will get converted into the result before hitting the runtime
    pxs_Factory,
    /// Exception is any exception happening at the language level. Pixel Script errors will be caught with pxs_Error in a future release
    pxs_Exception
}

/// A Factory variable data holder.
/// 
/// Holds a callback for creation. And the arguments to be supplied. 
/// Runtime will be supplied automatically.
#[allow(non_camel_case_types)]
pub struct pxs_FactoryHolder {
    pub callback: super::func::pxs_Func,
    pub args: *mut pxs_Var
}

impl pxs_FactoryHolder {
    /// Get a cloned list of args. This list will include the runtime passed as the
    /// first item. Returns a owned pxs_Var not a pointer.
    pub fn get_args(&self, rt: pxs_Runtime) -> pxs_Var {
        let args = self.args;
        // Check null
        assert!(!args.is_null(), "Factory args must not be null");
        unsafe {
            // Clone
            let args_clone = pxs_Var::from_borrow(self.args).clone();
            // Check list
            assert!(args_clone.is_list(), "Factory args must be a list");
            // Get list and add runtime
            let args_list = args_clone.get_list().unwrap();
            args_list.vars.insert(0, pxs_Var::new_i64(rt.into_i64()));

            args_clone
        }
    }

    /// Call the FactoryHolder function with the args!
    pub fn call(&self, rt: pxs_Runtime) -> pxs_Var {
        let args = self.get_args(rt);
        // Create raw memory that lasts for the direction of this call.
        let args_raw = args.into_raw();

        let res = unsafe {(self.callback)(args_raw, ptr::null_mut()) };
        let var = if res.is_null() {
            pxs_Var::new_null()
        } else {
            pxs_Var::from_raw(res)
        };
        // Drop raw args memory
        let _ = pxs_Var::from_raw(args_raw);

        // Return the variable result
        var
    }
}
// impl pxs_FactoryHolder {
//     /// Call the callback with args and null ptr
//     pub unsafe fn get_result(&mut self, rt: pxs_Runtime) -> pxs_VarT {
//         let args = unsafe{ pxs_Var::from_borrow(self.args) };
//         let list = args.get_list().unwrap();
//         if !self.has_rt {
//             pxs_debug!("Adding the runtime. Current length: {}", list.vars.len());
//             self.has_rt = true;
//             list.vars.insert(0, pxs_Var::new_i64(rt.into_i64()));
//         } else {
//             pxs_debug!("Resetting the runtime.");
//             list.set_item(pxs_Var::new_i64(rt.into_i64()), 0);
//         }
//         unsafe { (self.callback)(self.args, std::ptr::null_mut()) }
//     }
// }

impl PtrMagic for pxs_FactoryHolder {}

/// Holds data for a pxs_Var of list.
///
/// It holds multiple pxsVar within.
///
/// When creating call:
///
/// `pixelscript_var_newlist()`.
///
/// To add items
///
/// `pixelscript_var_list_add(list_ptr, item_ptr)`
///
/// To get items
///
/// `pixelscript_var_list_get(list_ptr, index)`
///
/// A full example looks like:
/// ```c
/// // Create a new list (you never interact with pxs_VarList directly...)
/// pxs_Var* list = pixelscript_var_newlist();
///
/// // Add a item
/// pxs_Var* number = pixelscript_var_newint(1);
/// pixelscript_var_list_add(list, number);
///
/// // Get a item
/// pxs_Var* item_got = pixelscript_var_list_get(list, 0);
/// ```
#[allow(non_camel_case_types)]
pub struct pxs_VarList {
    pub vars: Vec<pxs_Var>,
}

impl PtrMagic for pxs_VarList {}

impl pxs_VarList {
    /// Create a new VarList
    pub fn new() -> Self {
        pxs_VarList { vars: vec![] }
    }

    /// Add a Var to the list. List will take ownership.
    pub fn add_item(&mut self, item: pxs_Var) {
        self.vars.push(item);
    }

    /// Get a Var from the list. Supports negative based indexes.
    pub fn get_item(&self, index: i32) -> Option<&pxs_Var> {
        // Get correct negative index.
        let r_index = {
            if index < 0 {
                (self.vars.len() as i32) + index
            } else {
                index
            }
        };

        if r_index < 0 {
            None
        } else {
            self.vars.get(r_index as usize)
        }
    }

    /// Set at a specific index a item.
    ///
    /// Index must already be filled.
    pub fn set_item(&mut self, item: pxs_Var, index: i32) -> bool {
        // Get correct negative index.
        let r_index = {
            if index < 0 {
                (self.vars.len() as i32) + index
            } else {
                index
            }
        };

        if r_index < 0 {
            return false;
        }

        if self.vars.len() < r_index as usize {
            false
        } else {
            self.vars[r_index as usize] = item;
            true
        }
    }
}

/// The Variables actual value union.
#[repr(C)]
#[allow(non_camel_case_types)]
pub union pxs_VarValue {
    pub i64_val: i64,
    pub u64_val: u64,
    pub string_val: *mut c_char,
    pub bool_val: bool,
    pub f64_val: f64,
    pub null_val: *const c_void,
    pub object_val: *mut c_void,
    pub host_object_val: i32,
    pub list_val: *mut pxs_VarList,
    pub function_val: *mut c_void,
    pub factory_val: *mut pxs_FactoryHolder,
}

#[allow(non_camel_case_types)]
pub type pxs_DeleterFn = unsafe extern "C" fn(*mut c_void);

/// Default Var deleter fn.
/// Use it when you don't want to delete memory.
pub unsafe extern "C" fn default_deleter(_ptr: *mut c_void) {
    // Empty OP
}

/// A PixelScript Var(iable).
///
/// This is the universal truth between all languages PixelScript supports.
///
/// Currently supports:
/// - int (i32, i64, u32, u64)
/// - float (f32, f64)
/// - string
/// - boolean
/// - Objects
/// - HostObjects (C structs acting as pseudo-classes) This in the Host can also be a Int or Uint.
/// - List
/// - Functions (First class functions)
///
/// When working with objects you must use the C-api:
/// ```c
/// // Calls a method on a object.
/// pixelscript_object_call(var)
/// ```
///
/// When using within a callback, if said callback was attached to a Class, the first *mut Var will be the class/object.
///
/// When using ints or floats, if (i32, u32, u64, f32) there is no gurantee that the supported language uses
/// those types. Usually it defaults to i64 and f64.
///
/// When creating a object, this is a bit tricky but essentially you have to first create a pointer via the pixel script runtime.
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct pxs_Var {
    /// A tag for the variable type.
    pub tag: pxs_VarType,
    /// A value as a union.
    pub value: pxs_VarValue,

    /// Optional delete method. This is used for Pointers in Objects, and Functions.
    pub deleter: Cell<pxs_DeleterFn>,
}

// Rust specific functions
impl pxs_Var {
    pub unsafe fn slice_raw(argv: *mut *mut Self, argc: usize) -> &'static [*mut pxs_Var] {
        unsafe { std::slice::from_raw_parts(argv, argc) }
    }

    /// Get the direct host pointer. (Not the idx)
    pub fn get_host_ptr(&self) -> *mut c_void {
        let object = get_object(self.get_host_idx()).unwrap();
        object.ptr
    }

    /// Get the Rust string from the Var.
    /// 
    /// Works for pxs_String and pxs_Exception
    pub fn get_string(&self) -> Result<String, Error> {
        if self.tag == pxs_VarType::pxs_String || self.tag == pxs_VarType::pxs_Exception {
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

        pxs_Var {
            tag: pxs_VarType::pxs_String,
            value: pxs_VarValue {
                string_val: cstr.into_raw(),
            },
            deleter: Cell::new(default_deleter),
        }
    }

    /// Creates a new Null var.
    ///
    /// No need to free, or any of that. It cretes a *const c_void
    pub fn new_null() -> Self {
        pxs_Var {
            tag: pxs_VarType::pxs_Null,
            value: pxs_VarValue {
                null_val: ptr::null(),
            },
            deleter: Cell::new(default_deleter),
        }
    }

    /// Create a new HostObject var.
    pub fn new_host_object(ptr: i32) -> Self {
        pxs_Var {
            tag: pxs_VarType::pxs_HostObject,
            value: pxs_VarValue {
                host_object_val: ptr,
            },
            deleter: Cell::new(default_deleter),
        }
    }

    /// Create a new Object var.
    pub fn new_object(ptr: *mut c_void, deleter: Option<pxs_DeleterFn>) -> Self {
        let deleter = if let Some(d) = deleter {
            d
        } else {
            default_deleter
        };
        pxs_Var {
            tag: pxs_VarType::pxs_Object,
            value: pxs_VarValue { object_val: ptr },
            deleter: Cell::new(deleter),
        }
    }

    /// Create a new pxs_VarList var.
    pub fn new_list() -> Self {
        pxs_Var {
            tag: pxs_VarType::pxs_List,
            value: pxs_VarValue {
                list_val: pxs_VarList::new().into_raw(),
            },
            deleter: Cell::new(default_deleter),
        }
    }

    /// Create a new pxs_VarList var with values.
    pub fn new_list_with(vars: Vec<pxs_Var>) -> Self {
        let mut list = pxs_VarList::new();
        list.vars = vars;
        pxs_Var {
            tag: pxs_VarType::pxs_List,
            value: pxs_VarValue {
                list_val: list.into_raw(),
            },
            deleter: Cell::new(default_deleter),
        }
    }

    /// Create a new Function var.
    pub fn new_function(ptr: *mut c_void, deleter: Option<pxs_DeleterFn>) -> Self {
        let deleter = if let Some(d) = deleter {
            d
        } else {
            default_deleter
        };

        pxs_Var {
            tag: pxs_VarType::pxs_Function,
            value: pxs_VarValue { function_val: ptr },
            deleter: Cell::new(deleter),
        }
    }

    /// Create a new Factory var.
    pub fn new_factory(func: super::func::pxs_Func, args: pxs_VarT) -> Self {
        let factory = pxs_FactoryHolder {
            callback: func,
            args
        };
        pxs_Var {
            tag: pxs_VarType::pxs_Factory,
            value: pxs_VarValue {
                factory_val: factory.into_raw(),
            },
            deleter: Cell::new(default_deleter),
        }
    }

    /// Create a new Exception var.
    pub fn new_exception(msg: String) -> Self {
        pxs_Var {
            tag: pxs_VarType::pxs_Exception,
            value: pxs_VarValue {
                string_val: create_raw_string!(msg)
            },
            deleter: Cell::new(default_deleter)
        }
    }
    //     name: *const c_char,
    // func: pxs_Func,
    // args: *mut pxs_Var

    /// Get the IDX of the object if Host, i64, u64
    pub fn get_host_idx(&self) -> i32 {
        match self.tag {
            pxs_VarType::pxs_Int64 => self.get_i64().unwrap() as i32,
            pxs_VarType::pxs_UInt64 => self.get_u64().unwrap() as i32,
            pxs_VarType::pxs_HostObject => unsafe { self.value.host_object_val },
            _ => -1,
        }
    }

    /// Get the raw pointer to a `pxs_Object` type
    pub fn get_object_ptr(&self) -> *mut c_void {
        match self.tag {
            pxs_VarType::pxs_Object => unsafe {self.value.object_val },
            _ => std::ptr::null_mut()
        }
    }

    /// Get the pxs_VarList as a &mut pxs_VarList.
    pub fn get_list(&self) -> Option<&mut pxs_VarList> {
        if !self.is_list() {
            None
        } else {
            unsafe { Some(pxs_VarList::from_borrow(self.value.list_val)) }
        }
    }

    /// Get the pxs_FactoryHolder as a &mut pxs_FactoryHolder
    pub fn get_factory(&self) -> Option<&mut pxs_FactoryHolder> {
        if !self.is_factory() {
            None
        } else {
            unsafe {
                if self.value.factory_val.is_null() {
                    None
                } else {
                    Some(pxs_FactoryHolder::from_borrow(self.value.factory_val))
                }
            }
        }
    }

    ///
    pub unsafe fn from_argv(argc: usize, argv: *mut *mut pxs_Var) -> Vec<pxs_Var> {
        // First create a slice
        let argv_borrow = unsafe { pxs_Var::slice_raw(argv, argc) };
        // Now clone them!
        let cloned: Vec<pxs_Var> = argv_borrow
            .iter()
            .filter(|ptr| !ptr.is_null())
            .map(|&ptr| unsafe { (*ptr).clone() })
            .collect();

        cloned
    }

    /// Debug struct
    unsafe fn dbg(&self) -> String {
        unsafe {
            let details = match self.tag {
                pxs_VarType::pxs_Int64 => self.value.i64_val.to_string(),
                pxs_VarType::pxs_UInt64 => self.value.u64_val.to_string(),
                pxs_VarType::pxs_String => borrow_string!(self.value.string_val).to_string(),
                pxs_VarType::pxs_Bool => self.value.bool_val.to_string(),
                pxs_VarType::pxs_Float64 => self.value.f64_val.to_string(),
                pxs_VarType::pxs_Null => "Null".to_string(),
                pxs_VarType::pxs_Object => "Object".to_string(),
                pxs_VarType::pxs_HostObject => {
                    let idx = self.get_host_idx();
                    let object = get_object(idx).unwrap();
                    object.type_name.to_string()
                }
                pxs_VarType::pxs_List => {
                    let list = self.get_list().unwrap();
                    let t: String = list.vars.iter().map(|v| format!("{},", v.dbg())).collect();
                    format!("[{t}]")
                }
                pxs_VarType::pxs_Function => "Function".to_string(),
                pxs_VarType::pxs_Factory => "Factory".to_string(),
                pxs_VarType::pxs_Exception => borrow_string!(self.value.string_val).to_string()
            };

            format!("{details} :: {:p}", self)
        }
    }

    // /// Remove the deleter on current pxs_Var.
    // pub fn remove_deleter(mut self) -> Self {
    //     self.deleter = Cell::new(default_deleter);
    //     self
    // }

    write_func!(
        (get_i64, i64_val, i64, pxs_VarType::pxs_Int64),
        (get_u64, u64_val, u64, pxs_VarType::pxs_UInt64),
        (get_bool, bool_val, bool, pxs_VarType::pxs_Bool),
        (get_f64, f64_val, f64, pxs_VarType::pxs_Float64),
        (
            get_function,
            function_val,
            *mut c_void,
            pxs_VarType::pxs_Function
        )
    );

    // $t:ty, $func:ident, $vt:expr, $vn:ident
    write_new_methods! {
        i64, new_i64, pxs_VarType::pxs_Int64, i64_val;
        u64, new_u64, pxs_VarType::pxs_UInt64, u64_val;
        f64, new_f64, pxs_VarType::pxs_Float64, f64_val;
        bool, new_bool, pxs_VarType::pxs_Bool, bool_val
    }

    write_is_methods! {
        is_i64, pxs_VarType::pxs_Int64;
        is_u64, pxs_VarType::pxs_UInt64;
        is_f64, pxs_VarType::pxs_Float64;
        is_bool, pxs_VarType::pxs_Bool;
        is_string, pxs_VarType::pxs_String;
        is_null, pxs_VarType::pxs_Null;
        is_object, pxs_VarType::pxs_Object;
        is_host_object, pxs_VarType::pxs_HostObject;
        is_list, pxs_VarType::pxs_List;
        is_function, pxs_VarType::pxs_Function;
        is_factory, pxs_VarType::pxs_Factory;
        is_exception, pxs_VarType::pxs_Exception
    }
}

unsafe impl Send for pxs_Var {}
unsafe impl Sync for pxs_Var {}

impl Drop for pxs_Var {
    fn drop(&mut self) {
        // pxs_debug!("|psx_Var DROP| {}", unsafe {self.dbg() });
        if self.tag == pxs_VarType::pxs_String || self.tag == pxs_VarType::pxs_Exception {
            unsafe {
                // Free the mem
                if !self.value.string_val.is_null() {
                    let _ = CString::from_raw(self.value.string_val);
                    self.value.string_val = ptr::null_mut();
                }
            }
        } else if self.tag == pxs_VarType::pxs_List {
            let _ = unsafe {
                // This will automatically drop
                pxs_VarList::from_raw(self.value.list_val)
            };
        } else if self.tag == pxs_VarType::pxs_Object {
            unsafe {
                if self.value.object_val.is_null() {
                    return;
                }
                (self.deleter.get())(self.value.object_val)
            };
        } else if self.tag == pxs_VarType::pxs_Function {
            unsafe {
                if self.value.object_val.is_null() {
                    return;
                }
                (self.deleter.get())(self.value.object_val)
            };
        } else if self.tag == pxs_VarType::pxs_Factory {
            // Free args
            unsafe {
                if self.value.factory_val.is_null() {
                    return;
                }
                let val = pxs_FactoryHolder::from_borrow(self.value.factory_val);
                if val.args.is_null() {
                    return;
                }
                // Drop list
                let _ = pxs_Var::from_raw(val.args);
            }
        }
    }
}

impl PtrMagic for pxs_Var {}

impl Clone for pxs_Var {
    fn clone(&self) -> Self {
        unsafe {
            match self.tag {
                pxs_VarType::pxs_Int64 => pxs_Var::new_i64(self.value.i64_val),
                pxs_VarType::pxs_UInt64 => pxs_Var::new_u64(self.value.u64_val),
                pxs_VarType::pxs_String => {
                    let string = borrow_string!(self.value.string_val);
                    let cloned_string = string.to_string().clone();
                    let new_string = create_raw_string!(cloned_string);
                    pxs_Var {
                        tag: pxs_VarType::pxs_String,
                        value: pxs_VarValue {
                            string_val: new_string,
                        },
                        deleter: Cell::new(default_deleter),
                    }
                }
                pxs_VarType::pxs_Bool => pxs_Var::new_bool(self.value.bool_val),
                pxs_VarType::pxs_Float64 => pxs_Var::new_f64(self.value.f64_val),
                pxs_VarType::pxs_Null => pxs_Var::new_null(),
                pxs_VarType::pxs_Object => {
                    let r = pxs_Var {
                        tag: pxs_VarType::pxs_Object,
                        value: pxs_VarValue {
                            object_val: self.value.object_val,
                        },
                        deleter: Cell::new(self.deleter.get()),
                    };

                    self.deleter.set(default_deleter);

                    r
                }
                pxs_VarType::pxs_HostObject => pxs_Var::new_host_object(self.value.host_object_val),
                pxs_VarType::pxs_List => {
                    let mut list = pxs_VarList::new();
                    // let mut list = pxs_Var::new_list();
                    let og_list_val = pxs_VarList::from_borrow(self.value.list_val);

                    // Add items of current list. i.e. transfer ownership...
                    for item in og_list_val.vars.iter() {
                        // Clone into new list
                        list.add_item(item.clone());
                    }

                    pxs_Var {
                        tag: pxs_VarType::pxs_List,
                        value: pxs_VarValue {
                            list_val: list.into_raw(),
                        },
                        deleter: Cell::new(default_deleter),
                    }
                }
                pxs_VarType::pxs_Function => {
                    let r = pxs_Var {
                        tag: pxs_VarType::pxs_Function,
                        value: pxs_VarValue {
                            function_val: self.value.function_val,
                        },
                        deleter: Cell::new(self.deleter.get()),
                    };

                    self.deleter.set(default_deleter);
                    r
                }
                pxs_VarType::pxs_Factory => {
                    let og = pxs_FactoryHolder::from_borrow(self.value.factory_val);
                    if og.args.is_null() {
                        return pxs_Var::new_null();
                    }
                    // Clone args
                    let new_args = pxs_Var::new_list();
                    let old_args = pxs_Var::from_borrow(og.args);
                    if !old_args.is_list() {
                        return pxs_Var::new_null();
                    }
                    let old_list = old_args.get_list().unwrap();
                    let new_list = new_args.get_list().unwrap();
                    for var in old_list.vars.iter() {
                        new_list.add_item(var.clone());
                    }
                    let f = pxs_FactoryHolder {
                        args: new_args.into_raw(),
                        callback: og.callback
                    };
                    pxs_Var {
                        tag: pxs_VarType::pxs_Factory,
                        value: pxs_VarValue {
                            factory_val: f.into_raw(),
                        },
                        deleter: Cell::new(default_deleter),
                    }
                }
                pxs_VarType::pxs_Exception => {
                    let string = borrow_string!(self.value.string_val);
                    let cloned_string = string.to_string().clone();
                    let new_string = create_raw_string!(cloned_string);
                    pxs_Var {
                        tag: pxs_VarType::pxs_Exception,
                        value: pxs_VarValue {
                            string_val: new_string,
                        },
                        deleter: Cell::new(default_deleter),
                    }
                }
            }
        }
    }
}

impl std::fmt::Debug for pxs_Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debug_val: &dyn std::fmt::Debug = unsafe { &self.dbg() };
        f.debug_struct("pxs_Var")
            .field("tag", &self.tag)
            .field("value", debug_val)
            .finish()
    }
}

/// Methods for interacting with objects and callbacks from the runtime.
pub trait ObjectMethods {
    /// Call a method on a object.
    fn object_call(var: &pxs_Var, method: &str, args: &mut pxs_VarList) -> Result<pxs_Var, Error>;

    /// Call a method and pass in args
    fn call_method(method: &str, args: &mut pxs_VarList) -> Result<pxs_Var, Error>;

    /// Call a pxs_Var function.
    fn var_call(method: &pxs_Var, args: &mut pxs_VarList) -> Result<pxs_Var, Error>;

    /// Getter
    fn get(var: &pxs_Var, key: &str) -> Result<pxs_Var, Error>;

    /// Setter
    fn set(var: &pxs_Var, key: &str, value: &pxs_Var) -> Result<pxs_Var, Error>;
}

/// Type Helper for a pxs_Var
/// Use this instead of writing out pxs_Var*
#[allow(non_camel_case_types)]
pub type pxs_VarT = *mut pxs_Var;
