use std::sync::Arc;

use crate::{borrow_string, js::{JSModuleMethod, create_callback, get_js_state, quickjs}, shared::{module::pxs_Module, utils::CStringSafe}};

/// Module definition function
unsafe extern "C" fn init_module_function(ctx: *mut quickjs::JSContext, m: *mut quickjs::JSModuleDef) -> i32 {
    // Get module name.
    let module_name = unsafe {
        // Name ATOM
        let mna = quickjs::JS_GetModuleName(ctx, m);
        // Name ptr
        let mnp = quickjs::JS_AtomToCStringLen(ctx, std::ptr::null_mut(), mna);

        borrow_string!(mnp).to_string()
    };

    // Get state
    let state = get_js_state();
    
    // Set methods
    if let Some(methods) = state.module_functions.borrow().get(&module_name) {
        // set
        let mut cstrsafe = CStringSafe::new();
        for method in methods {
            unsafe {
                quickjs::JS_SetModuleExport(ctx, m, cstrsafe.new_string(&method.name), method.value.value);
            }
        }
    }

    0
}

/// Add a module to JS!
pub(super) fn add_module(context: *mut quickjs::JSContext, module: Arc<pxs_Module>) {
    let mut cstrsafe = CStringSafe::new();

    // Set it up my man
    let js_mod = unsafe {
        quickjs::JS_NewCModule(context, cstrsafe.new_string(&module.name), Some(init_module_function))
    };

    let mut trampolines = vec![];
    // Create trampolines
    for method in module.callbacks.iter() {
        // Create method
        let cbk = create_callback(context, method.idx);
        trampolines.push(JSModuleMethod{
            name: method.name.clone(),
            value: cbk
        });
        // Add export
        unsafe {
            quickjs::JS_AddModuleExport(context, js_mod, cstrsafe.new_string(&method.name));
        }
    }

    // Save in state
    let state = get_js_state();
    let mut module_functions = state.module_functions.borrow_mut();
    module_functions.insert(module.name.clone(), trampolines);

    // Save module
    let mut modules = state.modules.borrow_mut();
    modules.insert(module.name.clone(), js_mod);
}