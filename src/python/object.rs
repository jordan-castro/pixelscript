// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::sync::Arc;

use crate::{
    create_raw_string, free_raw_string, pxs_debug, python::{
        add_new_defined_object, add_new_name_idx_fn, eval_py, exec_py, is_object_defined, make_private_prefix, pocketpy, pocketpy_bridge
    }, shared::object::pxs_PixelObject
};

/// Save the object function. Returns the name of the function.
fn save_object_function(name: &str, idx: i32, module_name: &str) -> String {
    add_new_name_idx_fn(name.to_string(), idx);

    // Create a private name
    let private_name: String = make_private_prefix("", format!("privatemethod{idx}").as_str());

    // C stuff
    let c_name = create_raw_string!(private_name.clone());
    let c_main = create_raw_string!(module_name);

    unsafe {
        let scope = pocketpy::py_getmodule(c_main);

        // if scope.is_null() {
        //     pxs_debug!("scope is null for : {module_name}");
        // }

        pocketpy::py_bindfunc(scope, c_name, Some(pocketpy_bridge));
        free_raw_string!(c_main);
    }

    private_name
}

/// Create a object type in the Python Runtime.
///
/// idx: is the saved object.
/// source: is the object methods
pub(super) fn create_object(idx: i32, source: Arc<pxs_PixelObject>, module_name: &str) {
    // pxs_debug!("create_object start for idx: {idx}, type_name: {} in moudule: {module_name}", source.type_name);
    let rmodule_name = module_name.to_string().clone();
    // Create the module if it does not already exist
    unsafe {
        let c_module_name = create_raw_string!(rmodule_name.clone());
        let pymodule = pocketpy::py_getmodule(c_module_name);
        if pymodule.is_null() {
            pocketpy::py_newmodule(c_module_name);
        }
    }

    let object_name = format!("{rmodule_name}{}", source.type_name).replace(".", "_");
    // Check if object is defined.
    let obj_exists = is_object_defined(&object_name);
    if obj_exists {
        // pxs_debug!("Object is alredy defined!");
        #[allow(unused)]
        let eval_err = eval_py(
            format!("_{}({})", object_name, idx).as_str(),
            format!("<create_{}>", &object_name).as_str(),
            module_name,
        );
        // if eval_err.len() > 0 {
            // TODO: use py_raise here
        // }
        return;
    }

    // Object does not exist
    // First register callbacks
    let mut methods_str = String::new();
    for method in source.callbacks.iter() {
        let method_name = format!("{}{}", object_name, method.cbk.name);
        // pxs_debug!("Adding method name: {method_name}");
        // let private_name = make_private(&method.name);
        let private_name = save_object_function(&method_name, method.cbk.idx, module_name);
        let input = if method.is_id {
            "._pxs_ptr"
        } else {
            ""
        };
        methods_str.push_str(
            format!(
                r#"
    def {}(self, *args):
        return {}('{}', self{}, *args)
        
"#,
                method.cbk.name, private_name, method_name, input
            )
            .as_str(),
        );
    }

    let object_string = format!(
        r#"
# Bridge for pocketpy
class _{}:
    def __init__(self, ptr):
        # Set the ptr
        self._pxs_ptr = ptr

{}
"#,
        object_name, methods_str
    );

    pxs_debug!("{object_string}");

    // Execute it
    let res = exec_py(
        &object_string,
        format!("<first_{}>", &object_name).as_str(),
        module_name,
    );
    if !res.is_empty() {
        return;
    }

    // add it
    add_new_defined_object(&object_name);

    // Ok but just create it now
    let res = eval_py(
        format!("_{}({})", object_name, idx).as_str(),
        format!("<create_{}>", object_name).as_str(),
        &rmodule_name,
    );
    if !res.is_empty() {
        println!("Python create_object error: {res}");
    }
}
