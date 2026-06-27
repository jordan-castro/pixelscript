// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::sync::Arc;

use etffi::cstring::CStringSafe;

use crate::{
    pxs_debug, python::{
        PXS_CALL_METHOD, add_new_defined_object, eval_py, exec_py, func::get_from_obj, is_object_defined, pocketpy, pocketpy_bridge
    }, shared::{object::{ObjectFlags, pxs_PixelObject}}
};

/// Create a object type in the Python Runtime.
///
/// idx: is the saved object.
/// source: is the object methods
pub(super) fn create_object(idx: i32, source: Arc<pxs_PixelObject>, module_name: &str) {
    // pxs_debug!("create_object start for idx: {idx}, type_name: {} in moudule: {module_name}", source.type_name);
    let rmodule_name = module_name.to_string().clone();
    let mut cstr_safe = CStringSafe::new();
    // Create the module if it does not already exist
    unsafe {
        let c_module_name = cstr_safe.new_string(&rmodule_name.clone());
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
        return;
    }

    // Define _pxs_call for this objects module.
    unsafe {
        let module = pocketpy::py_getmodule(cstr_safe.new_string(&rmodule_name.clone()));
        let func = get_from_obj(module, PXS_CALL_METHOD);
        if func.is_none() {
            // Assign function to module
            pocketpy::py_bindfunc(module, cstr_safe.new_string(PXS_CALL_METHOD), Some(pocketpy_bridge));
        }
    }

    // Object does not exist
    // First register callbacks
    let mut methods_str = String::new();
    for method in source.callbacks.iter() {
        // Check input type
        let input = if method.flags & ObjectFlags::UsesId as u8 != 0 {
            "._pxs_ptr"
        } else {
            ""
        };

        // Check for property
        if method.flags & ObjectFlags::IsProp as u8 != 0 {
            methods_str.push_str("\n    @property");
        }

        let function_string = format!(
            r#"
    def {}(self, *args):
        return {}({}, self{}, *args)
"#,
        method.cbk.name, PXS_CALL_METHOD, method.cbk.idx, input
        );
        methods_str.push_str(&function_string);

        if method.flags & ObjectFlags::IsProp as u8 != 0 {
            methods_str.push_str(format!("\n    @{}.setter", method.cbk.name).as_str());
            // Add function
            methods_str.push_str(&function_string);
        }
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

    pxs_debug!("{object_string} {module_name}");

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
        pxs_debug!("Python create_object error: {res}");
    }
}
