use std::sync::Arc;

use crate::{borrow_string, borrow_var, js::{JSModuleMethod, SmartJSValue, create_callback, get_js_state, pxs_into_js, quickjs}, pxs_debug, shared::{PtrMagic, module::pxs_Module, utils::CStringSafe, var::pxs_Var}};

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
    if let Some(exports) = state.module_exports.borrow().get(&module_name) {
        // set
        let mut cstrsafe = CStringSafe::new();
        for export in exports {
            unsafe {
                quickjs::JS_SetModuleExport(ctx, m, cstrsafe.new_string(&export.name), export.value.value);
            }
        }
    }

    0
}

/// Add a module to JS!
pub(super) fn add_module(context: *mut quickjs::JSContext, module: &Arc<pxs_Module>) {
    let mut cstrsafe = CStringSafe::new();

    // Set it up my man
    let js_mod = unsafe {
        quickjs::JS_NewCModule(context, cstrsafe.new_string(&module.name), Some(init_module_function))
    };

    let mut exports = vec![];
    // Create trampolines
    for method in module.callbacks.iter() {
        // Create method
        let cbk = create_callback(context, method.idx);
        exports.push(JSModuleMethod{
            name: method.name.clone(),
            value: cbk
        });
        // Add export
        unsafe {
            quickjs::JS_AddModuleExport(context, js_mod, cstrsafe.new_string(&method.name));
        }
    }

    // Uh Uh Uh set variables
    for module_var in module.variables.iter() {
        // Create variables
        let var = pxs_into_js(context, borrow_var!(module_var.var));
        if let Err(err) = var {
            let err_msg = err.to_string();
            let mut exception = SmartJSValue::new_exception(context, err_msg, "PXSJSConversionError".to_string());
            exception.owned = false;
            exports.push(JSModuleMethod{
                name: module_var.name.clone(),
                value: exception
            });
        } else {
            let mut res = var.unwrap();
            res.owned = false;
            exports.push(JSModuleMethod { 
                name: module_var.name.clone(), 
                value: res, 
            });
        }

        // Add export
        unsafe {
            quickjs::JS_AddModuleExport(context, js_mod, cstrsafe.new_string(&module_var.name));
        }
    }

    // Save in state
    let state = get_js_state();
    let mut module_exports = state.module_exports.borrow_mut();
    module_exports.insert(module.name.clone(), exports);

    // Save module
    let mut modules = state.modules.borrow_mut();
    modules.insert(module.name.clone(), js_mod);

    drop(modules);
    drop(module_exports);
    drop(state);

    // Add child modules
    for child in module.modules.iter() {
        add_module(context, child);
    }
}

/// Add a local module to JS engine.
pub(super) fn add_local_module(context: *mut quickjs::JSContext, code: &str, name: &str) -> *mut quickjs::JSModuleDef {
    let mut cstrsafe = CStringSafe::new();

    // Compile module
    let smart_module = SmartJSValue::new_owned(unsafe {
        quickjs::JS_Eval(context, cstrsafe.new_string(code), code.len(), cstrsafe.new_string(name), (quickjs::JS_EVAL_TYPE_MODULE | quickjs::JS_EVAL_FLAG_COMPILE_ONLY) as i32)
    }, context);

    // Check exception
    if smart_module.is_exception() || smart_module.is_error() {
        pxs_debug!("Error compiling module");
        return std::ptr::null_mut();
    }

    smart_module.get_module_ptr()
}