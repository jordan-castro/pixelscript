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
    create_raw_string, free_raw_string,
    python::{
        add_new_defined_object, add_new_name_idx_fn, eval_py, exec_py, is_object_defined,
        make_private, pocketpy, pocketpy_bridge,
    },
    shared::object::PixelObject,
};

fn save_object_function(name: &str, idx: i32, module_name: &str) {
    add_new_name_idx_fn(name.to_string(), idx);

    // Create a private name
    let private_name = make_private(name);

    // C stuff
    let c_name = create_raw_string!(private_name.clone());
    let c_main = create_raw_string!("__main__");
    let bridge_code = format!(
        r#"
def {name}(*args):
    return {private_name}('{name}', *args)
"#
    );
    let c_brige_name = format!("<callback_bridge for {private_name}>");
    unsafe {
        let global_scope = pocketpy::py_getmodule(c_main);

        pocketpy::py_bindfunc(global_scope, c_name, Some(pocketpy_bridge));

        // Execute bridge
        let _s = exec_py(&bridge_code, &c_brige_name, module_name);
        free_raw_string!(c_name);
        free_raw_string!(c_main);
    }
}

/// Create a object type in the Python Runtime.
///
/// idx: is the saved object.
/// source: is the object methods
pub(super) fn create_object(idx: i32, source: Arc<PixelObject>, module_name: &str) {
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

    // Object does not exist
    // First register callbacks
    let mut methods_str = String::new();
    for method in source.callbacks.iter() {
        let private_name = make_private(&method.name);
        save_object_function(&method.name, method.idx, module_name);
        methods_str.push_str(
            format!(
                r#"
    def {}(self, *args):
        return {}('{}', self.ptr, *args)
        
"#,
                method.name, private_name, method.name
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
        self.ptr = ptr

{}

"#,
        source.type_name, methods_str
    );

    // _{}({})
    // , source.type_name, idx
    // println!("{object_string}");

    // Execute it
    let res = exec_py(
        &object_string,
        format!("<first_{}>", &source.type_name).as_str(),
        module_name,
    );
    if !res.is_empty() {
        return;
    }

    // add it
    add_new_defined_object(&source.type_name);

    // Ok but just create it now
    let res = eval_py(
        format!("_{}({})", source.type_name, idx).as_str(),
        "<create_{}>",
        module_name,
    );
    if !res.is_empty() {
        println!("PYTHONSDNAOSDNOIDRes: {res}");
    }
}
