// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::{
    python::{
        PYTHON_PRIVATE_METHOD, exec_py, pocketpy, pocketpy_bridge, var_to_pocketpyref
    },
    shared::{module::pxs_Module, utils::CStringSafe},
};

pub(super) fn create_module(module: &pxs_Module) {
    // Get module name
    let module_name = module.name.clone();
    let mut cstr_safe = CStringSafe::new();

    // Create module
    let c_module_name = cstr_safe.new_string(&module_name.clone());
    let pymodule = unsafe {
        // Check first if module already exists... (in the case of a variable object)
        let posmodule = pocketpy::py_getmodule(c_module_name);
        if posmodule.is_null() {
            pocketpy::py_newmodule(c_module_name)
        } else {
            posmodule
        }
    };

    // Add variables to module
    for var in module.variables.iter() {
        let var_name = var.name.clone();
        let c_var_name = cstr_safe.new_string(&var_name);
        let tmp = unsafe { pocketpy::py_pushtmp() };
        // pxs_debug!("|MODULEVARIABLE| {}", var.name);
        var_to_pocketpyref(
            tmp,
            &var.var,
            Some(&module_name),
        );

        // Set
        unsafe {
            let py_name = pocketpy::py_name(c_var_name);
            pocketpy::py_setattr(pymodule, py_name, tmp);
        }
    }

    // Bind a single function for the module
    let module_function_name = PYTHON_PRIVATE_METHOD;

    unsafe {
        pocketpy::py_bindfunc(pymodule, cstr_safe.new_string(module_function_name), Some(pocketpy_bridge));
    }

    let mut methods = String::new();

    // Add callbacks to module... This also needs to go through the pybridge
    for method in module.callbacks.iter() {
        let bridge_code = format!(
            r#"
def {}(*args):
    return {module_function_name}({}, *args)
"#,
            method.name, method.idx
        );
        methods.push_str(&bridge_code);
    }

    // Run bridge_code in current module
    exec_py(
        &methods,
        format!("<{}>", module_name).as_str(),
        &module_name,
    );

    // Do the same for internal modules
    for im in module.modules.iter() {
        create_module(im);
    }
}
