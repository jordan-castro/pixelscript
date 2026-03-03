// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::{pxs_debug, shared::{PtrMagic, func::pxs_Func, var::pxs_Var}};
use std::{backtrace::Backtrace, sync::Arc};

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
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct pxs_Module {
    /// Name of the module.
    pub name: String,
    /// Callbacks that need to be added.
    pub callbacks: Vec<ModuleCallback>,
    /// Variables that need to be added.
    pub variables: Vec<ModuleVariable>,
    /// Internal modules
    pub modules: Vec<Arc<pxs_Module>>,
    /// Factory variables
    pub factories: Vec<ModuleFactoryVariable>
}

/// Wraps a idx with a name.
#[derive(Clone)]
pub struct ModuleCallback {
    pub name: String,
    pub full_name: String,
    pub idx: i32,
}

/// Wraps a Var with a name.
#[derive(Clone)]
pub struct ModuleVariable {
    pub name: String,
    pub var: *mut pxs_Var,
}

/// Wraps a Var with a name and a value from a callback.
/// Basically a Factory call.
#[derive(Clone)]
pub struct ModuleFactoryVariable {
    pub name: String,
    pub callback: pxs_Func,
    pub args: *mut pxs_Var
}

impl pxs_Module {
    /// Create a new module.
    pub fn new(name: String) -> Self {
        pxs_Module {
            name: name,
            callbacks: vec![],
            variables: vec![],
            modules: vec![],
            factories: vec![]
        }
    }

    /// Add a callback to current module.
    pub fn add_callback(&mut self, name: &str, full_name: &str, idx: i32) {
        self.callbacks.push(ModuleCallback {
            name: name.to_string(),
            full_name: full_name.to_string(),
            idx,
        });
    }

    /// Add a variable to current module.
    pub fn add_variable(&mut self, name: &str, var: *mut pxs_Var) {
        self.variables.push(ModuleVariable {
            name: name.to_string(),
            var: var,
        });
    }

    /// Add a internal module.
    pub fn add_module(&mut self, child: Arc<pxs_Module>) {
        self.modules.push(child);
    }

    /// Get name without package
    pub fn get_name(&self) -> String {
        if !self.name.contains(".") {
            self.name.clone()
        } else {
            self.name.split(".").last().unwrap().to_string()
        }
    }

    /// Get package name
    pub fn get_pkg(&self) -> String {
        if !self.name.contains(".") {
            String::new()
        } else {
            let name = self.get_name();
            self.name.replace(&format!(".{name}"), "")
        }
    }

    /// Add a Factory variable
    pub fn add_factory_variable(&mut self, name: String, func: pxs_Func, args: *mut pxs_Var) {
        self.factories.push(ModuleFactoryVariable { name, callback: func, args });
    }
}

impl PtrMagic for pxs_Module {}

unsafe impl Send for pxs_Module {}
unsafe impl Sync for pxs_Module {}

unsafe impl Send for ModuleCallback {}
unsafe impl Sync for ModuleCallback {}

unsafe impl Send for ModuleVariable {}
unsafe impl Sync for ModuleVariable {}

impl Drop for pxs_Module {
    fn drop(&mut self) {
        pxs_debug!("Drop triggered for Module: {}", self.name);
        // pxs_debug!("Stack trace: {}", Backtrace::force_capture());

        for var in self.variables.drain(0..self.variables.len()) {
            if var.var.is_null() {
                continue;
            }
            // Drop the variable.
            let _ = pxs_Var::from_raw(var.var);
        }
    }
}
