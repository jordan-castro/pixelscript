// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{fmt::format, sync::Arc};

use crate::{
    create_raw_string, free_raw_string, pxs_debug, python::{
        add_new_defined_object, add_new_name_idx_fn, eval_py, exec_py, is_object_defined, make_private_prefix, pocketpy, pocketpy_bridge
    }, shared::object::pxs_PixelObject
};

/// Save the object function. Returns the name of the function.
fn save_object_function(name: &str, idx: i32, module_name: &str) -> String {
    add_new_name_idx_fn(name.to_string(), idx);

    // Create a private name
    let private_name: String = make_private_prefix(name, format!("{module_name}{idx}").as_str());

    // C stuff
    let c_name = create_raw_string!(private_name.clone());
    let c_main = create_raw_string!(module_name);
    let bridge_code = format!(
        r#"
def {name}(*args):
    return {private_name}('{name}', *args)
"#
    );
    let c_brige_name = format!("<callback_bridge for {private_name}>");
    unsafe {
        let scope = pocketpy::py_getmodule(c_main);

        pocketpy::py_bindfunc(scope, c_name, Some(pocketpy_bridge));

        // Execute bridge
        let res = exec_py(&bridge_code, &c_brige_name, module_name);
        if res.len() != 0 {
            pxs_debug!("python error save_object_function: {res}");
        }
        free_raw_string!(c_name);
        free_raw_string!(c_main);
    }

    private_name
}

/// Create a object type in the Python Runtime.
///
/// idx: is the saved object.
/// source: is the object methods
pub(super) fn create_object(idx: i32, source: Arc<pxs_PixelObject>, module_name: &str) {
    pxs_debug!("module name is: {module_name}");
    let rmodule_name = module_name.to_string().clone();
    // Check if object is defined.
    let obj_exists = is_object_defined(&source.type_name);
    if obj_exists {
        eval_py(
            format!("_{}({})", source.type_name, idx).as_str(),
            format!("<create_{}>", &source.type_name).as_str(),
            module_name,
        );
        return;
    }
    pxs_debug!("module name is: {module_name} 2");

    // Object does not exist
    // First register callbacks
    let mut methods_str = String::new();
    for method in source.callbacks.iter() {
        let method_name = format!("{}{}", source.type_name, method.name);
        // let private_name = make_private(&method.name);
        let private_name = save_object_function(&method_name, method.idx, module_name);
        methods_str.push_str(
            format!(
                r#"
    def {}(self, *args):
        return {}('{}', self._pxs_ptr, *args)
        
"#,
                method.name, private_name, method_name
            )
            .as_str(),
        );
    }
    pxs_debug!("module name is: {module_name} 3");

    let object_string = format!(
        r#"
# Bridge for pocketpy
class _{}:
    def __init__(self, ptr):
        # Set the ptr
        self._pxs_ptr = ptr

{}
"#,
        source.type_name, methods_str
    );

    pxs_debug!("{object_string}");
    pxs_debug!("module name is: {module_name} 4");

    // Execute it
    let res = exec_py(
        &object_string,
        format!("<first_{}>", &source.type_name).as_str(),
        module_name,
    );
    if !res.is_empty() {
        return;
    }
    pxs_debug!("module name is: {module_name} 5");


    // add it
    add_new_defined_object(&source.type_name);

    // Ok but just create it now
    let res = eval_py(
        format!("_{}({})", source.type_name, idx).as_str(),
        format!("<create_{}>", source.type_name).as_str(),
        &rmodule_name,
    );
    if !res.is_empty() {
        println!("Python create_object error: {res}");
    }
}
