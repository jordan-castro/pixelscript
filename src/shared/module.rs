use std::ffi::c_void;

use crate::shared::{
    PtrMagic,
    func::{Func, Function},
    var::Var,
};

/// A Module is a C representation of data that needs to be (imported,required, etc)
///
/// The process is you add callbacks, variables, etc.
///
/// And THEN add the module.
///
/// So you first need to call
///
/// pixelmods_create_module() Which will create a new module struct with a name.
///
/// Here is a simple example.
///
/// ```c
/// Module* m = pixelmods_new_module("math");
///
/// pixelmods_module_add_callback(m, ...);
/// pixelmods_module_add_variable(m, ...);
///
/// pixelmods_add_module(m);
/// ```
///
/// You never free the module pointer because the runtime takes ownership.
///
/// Callbacks within modules use the same FUNCTION_LOOKUP global static variable.
pub struct Module {
    /// Name of the module.
    pub name: String,
    /// Callbacks that need to be added.
    pub callbacks: Vec<ModuleCallback>,
    /// Variables that need to be added.
    pub variables: Vec<ModuleVariable>,
    /// Internal modules
    pub modules: Vec<Module>,
    /// Objects assigned to this Module
    pub objects: Vec<ModuleCallback>,
}

/// Wraps a Func with a name.
pub struct ModuleCallback {
    pub name: String,
    pub func: Function,
}

/// Wraps a Var with a name.
pub struct ModuleVariable {
    pub name: String,
    pub var: Var,
}

impl Module {
    /// Create a new module.
    pub fn new(name: String) -> Self {
        Module {
            name: name,
            callbacks: vec![],
            variables: vec![],
            modules: vec![],
            objects: vec![],
        }
    }

    /// Add a callback to current module.
    pub fn add_callback(&mut self, name: &str, func: Func, opaque: *mut c_void) {
        self.callbacks.push(ModuleCallback {
            name: name.to_string(),
            func: Function { func, opaque },
        });
    }

    /// Add a variable to current module.
    pub fn add_variable(&mut self, name: &str, var: &Var) {
        self.variables.push(ModuleVariable {
            name: name.to_string(),
            var: var.clone(),
        });
    }

    /// Add a internal module.
    pub fn add_module(&mut self, child: Module) {
        self.modules.push(child);
    }

    /// Add a internal object.
    pub fn add_object(&mut self, name: &str, constructor: Func, opaque: *mut c_void) {
        self.objects.push(ModuleCallback {
            name: name.to_owned(),
            func: Function {
                func: constructor,
                opaque,
            },
        });
    }
}

impl PtrMagic for Module {}

unsafe impl Send for Module {}
unsafe impl Sync for Module {}

unsafe impl Send for ModuleCallback {}
unsafe impl Sync for ModuleCallback {}

unsafe impl Send for ModuleVariable {}
unsafe impl Sync for ModuleVariable {}
