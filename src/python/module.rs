// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::{create_raw_string, free_raw_string, python::{add_new_name_idx_fn, exec_py, make_private, pocketpy, pocketpy_bridge, var_to_pocketpyref}, shared::{PtrMagic, module::pxs_Module, var::pxs_Var}};

pub(super) fn create_module(module: &pxs_Module, parent: Option<&str>) {
    // Get module name
    let module_name = match parent {
        Some(s) => format!("{s}.{}", module.name),
        None => module.name.clone(),
    };

    // Create module
    let c_module_name = create_raw_string!(module_name.clone());
    let pymodule = unsafe { pocketpy::py_newmodule(c_module_name) };

    // Add variables to module
    for var in module.variables.iter() {
        let var_name = var.name.clone();
        let c_var_name = create_raw_string!(var_name);
        let tmp = unsafe { pocketpy::py_pushtmp() };
        var_to_pocketpyref(tmp, unsafe{pxs_Var::from_borrow(var.var)});
        
        // Set
        unsafe {
            let py_name = pocketpy::py_name(c_var_name);
            pocketpy::py_setattr(pymodule, py_name, tmp);
            free_raw_string!(c_var_name);
        }
    }
    
    // Add callbacks to module... This also needs to go through the pybridge
    for method in module.callbacks.iter() {
        let full_name = method.full_name.clone();
        // Save function
        add_new_name_idx_fn(full_name.clone(), method.idx);

        // Private name
        let private_name = make_private(&full_name);

        let c_name = create_raw_string!(private_name.clone());
        let bridge_code = format!(r#"
def {}(*args):
    return {private_name}('{}', *args)
"#, method.name, full_name);

        // Register pocketpy_bridge
        unsafe {
            pocketpy::py_bindfunc(pymodule, c_name, Some(pocketpy_bridge));
        }

        // Run bridge_code in current module
        exec_py(&bridge_code, format!("<{}>", module_name).as_str(), &module_name);
        
        // Free c
        unsafe {
            free_raw_string!(c_name);
        }
    }

    // Do the same for internal modules
    for im in module.modules.iter() {
        create_module(im, Some(&module_name));
    }

    unsafe {
        free_raw_string!(c_module_name);
    }
}
