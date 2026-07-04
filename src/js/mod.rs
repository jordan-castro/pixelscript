use std::collections::HashMap;

use etffi::{
    borrow_string, create_raw_string,
    cstring::CStringSafe,
    free_raw_string,
    ptr_magic::{PtrMagic, ThreadSafePointer},
};

use crate::{
    js::{
        func::create_callback,
        module::add_local_module,
        utils::SmartJSValue,
        var::{js_into_pxs, pxs_into_js},
    }, pxs_debug, pxs_error, shared::{
        PXS_METHOD_NAME, PixelScript, PxsRes, PxsResult, pxs_Opaque, read_file,
        var::{ObjectMethods, pxs_Var},
    }, with_feature,
};

// Allow for the binidngs only
#[allow(unused)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub(self) mod quickjs {
    include!(concat!(env!("OUT_DIR"), "/quickjsng_bindings.rs"));
}

mod func;
mod module;
mod object;
mod utils;
mod var;

/// Holds name and Value for module methods
pub(self) struct JSModuleMethod {
    pub name: String,
    pub value: SmartJSValue,
}

/// JS specific State.
struct State {
    /// The JS runtime. Each thread gets it's own runtime.
    rt: *mut quickjs::JSRuntime,
    /// The `__main__` context. Each thread gets it's own context.
    context: *mut quickjs::JSContext,
    /// Keep a list of defined PixelObject as class
    defined_objects: HashMap<String, SmartJSValue>,
    /// Module defined functions or variables takes a map[module_name] => map[int] => export
    module_exports: HashMap<String, Vec<JSModuleMethod>>,
    /// JSModules
    modules: HashMap<String, *mut quickjs::JSModuleDef>,
}

/// Creates a raw pointer with empty values
fn new_state() -> *mut State {
    State {
        rt: std::ptr::null_mut(),
        context: std::ptr::null_mut(),
        defined_objects: HashMap::new(),
        module_exports: HashMap::new(),
        modules: HashMap::new(),
    }
    .into_raw()
}

/// Initialize the state.
fn init(ptr: *mut State) {
    unsafe {
        let rt = quickjs::JS_NewRuntime();
        let ctx = quickjs::JS_NewContext(rt);

        (*ptr).rt = rt;
        (*ptr).context = ctx;

        // Setup module loader!
        quickjs::JS_SetModuleLoaderFunc(rt, None, Some(js_module_loader), std::ptr::null_mut());

        with_feature!("pxs_json", {
            // Add pxs_json.js
            let pxs_json = add_local_module(ctx, include_str!("../../core/js/pxs_json.js"), "pxs_json");

            (*ptr).modules.insert("pxs_json".to_string(), pxs_json);
        });

        with_feature!("js_commonjs", {
            let require_name = create_raw_string!("require");
            // add commonJS.`require`
            let mut require_func = SmartJSValue::new_owned(quickjs::JS_NewCFunction2(
                ctx,
                Some(commonjs_require),
                require_name,
                "require".len() as i32,
                quickjs::JSCFunctionEnum_JS_CFUNC_generic,
                0,
            ), ctx);
            free_raw_string!(require_name);
            let globals = SmartJSValue::globalThis(ctx);
            globals.set_prop("require", &mut require_func);
        });

        add_main_js();
    }
}

/// Clear the State
fn clear(ptr: *mut State) {
    import_all_modules();
    unsafe {
        (*ptr).defined_objects.clear();
        (*ptr).module_exports.clear();
        (*ptr).modules.clear();
        if !(*ptr).context.is_null() {
            quickjs::JS_FreeContext((*ptr).context);
        }
        if !(*ptr).rt.is_null() {
            quickjs::JS_FreeRuntime((*ptr).rt);
        }
        (*ptr).context = std::ptr::null_mut();
        (*ptr).rt = std::ptr::null_mut();
    }
}

impl PtrMagic for State {}

impl Drop for State {
    fn drop(&mut self) {
        if !self.rt.is_null() {
            panic!("JS State must be freed before dropping.");
        }
    }
}

thread_local! {
    static JSTATE: ThreadSafePointer<State> = ThreadSafePointer::new_owned(new_state());
}

/// JS Module loader
unsafe extern "C" fn js_module_loader(
    context: *mut quickjs::JSContext,
    module_name: *const std::ffi::c_char,
    _opaque: pxs_Opaque,
) -> *mut quickjs::JSModuleDef {
    let state = get_js_state();
    unsafe {
        // let modules = (*state).modules.borrow();
        let name = borrow_string!(module_name);
        if let Some(module) = (*state).modules.get(name) {
            return *module;
        }

        // Otherwise try to read the file...
        let contents = read_file(name);
        if contents.len() == 0 {
            return std::ptr::null_mut();
        }

        let mut cstrsafe = CStringSafe::new();
        // We need to evalute a module
        let res = quickjs::JS_Eval(
            context,
            cstrsafe.new_string(&contents),
            contents.len(),
            module_name,
            (quickjs::JS_EVAL_TYPE_MODULE | quickjs::JS_EVAL_FLAG_COMPILE_ONLY) as i32,
        );
        let smart_res = SmartJSValue::new_borrow(res, context);

        // Check exception
        if smart_res.is_exception() || smart_res.is_error() {
            pxs_debug!("Error compiling module");
            return std::ptr::null_mut();
        }

        let val_int = smart_res.value.u.ptr as isize;
        let m = ((val_int & !15) as *mut std::ffi::c_void).cast::<quickjs::JSModuleDef>();

        m
    }
}

fn get_js_state() -> *mut State {
    JSTATE.with(|mutex| mutex.get_ptr())
}

#[cfg(feature = "js_commonjs")]
/// Add commonJS `require`
unsafe extern "C" fn commonjs_require(
    context: *mut quickjs::JSContext,
    _this_val: quickjs::JSValue,
    argc: i32,
    argv: *mut quickjs::JSValue,
) -> quickjs::JSValue {
    let mut cstring = CStringSafe::new();
    unsafe {
        // Check we have at least 1 arg (the path.)
        if argc < 1 {
            return quickjs::JS_ThrowInternalError(
                context,
                cstring.new_string("require requires 1 argument."),
            );
        }
        // Get the module name as a string.
        let module_name_val = SmartJSValue::new_borrow(*argv.offset(0), context);
        if !module_name_val.is_string() {
            return SmartJSValue::new_exception(
                context,
                "Expected module name to be a string.".to_string(),
                "TypeException".to_string(),
            )
            .dupped_value();
        }
        let module_name = module_name_val.as_string().unwrap();
        // Load the actual module using our internal module loader.
        let module = js_module_loader(
            context,
            cstring.new_string(&module_name),
            core::ptr::null_mut(),
        );
        if module.is_null() {
            return SmartJSValue::new_exception(
                context,
                format!("Could not find or compile module `{module_name}`."),
                "LoadModuleException".to_string(),
            )
            .dupped_value();
        }

        // Evaluate it again or for the first time!
        let code = format!("import * as __{module_name}__ from '{module_name}';");
        let res = run_js(&code, "<commonjs_require>", quickjs::JS_EVAL_TYPE_MODULE as i32);
        if res.is_exception() {
            return res.dupped_value();
        }

        // Ok we have module, and it is evaluated ::: lets get namespace
        let namespace = quickjs::JS_GetModuleNamespace(context, module);
        namespace
    }
}

/// Get context of state
pub(self) fn get_context(state: *mut State) -> *mut quickjs::JSContext {
    unsafe { (*state).context }
}

/// Run JS code
fn run_js(code: &str, file_name: &str, eval_type: i32) -> SmartJSValue {
    let mut cstrsafe = CStringSafe::new();
    let state = get_js_state();
    unsafe {
        let val = quickjs::JS_Eval(
            get_context(state),
            cstrsafe.new_string(code),
            code.len(),
            cstrsafe.new_string(file_name),
            eval_type,
        );

        // Check for exception
        let exception = SmartJSValue::current_exception(get_context(state));
        if exception.is_undefined() {
            let smart = SmartJSValue::new_owned(val, get_context(state));
            if smart.is_promise() {
                smart.await_value()
            } else {
                smart
            }
        } else {
            exception
        }
    }
}

/// Get JS Name (runs code without global this)
fn get_js_name(name: &str) -> SmartJSValue {
    run_js(name, "<get_js_name>", quickjs::JS_EVAL_TYPE_GLOBAL as i32)
}

/// Add main.js
fn add_main_js() {
    run_js(
        include_str!("../../core/js/main.js"),
        "main.js",
        quickjs::JS_EVAL_TYPE_MODULE as i32,
    );
}

/// Import all modules to initialize the app
fn import_all_modules() {
    let state = get_js_state();
    // let modules = state.modules.borrow();
    let mut import_modules_code = String::new();
    unsafe {
        for m in (*state).modules.iter() {
            import_modules_code.push_str(format!("import '{}';", m.0).as_str());
        }
    }
    run_js(
        &import_modules_code,
        "<cleanup>",
        quickjs::JS_EVAL_TYPE_MODULE as i32,
    );
}

pub struct JSScripting;

impl PixelScript for JSScripting {
    fn start() {
        init(get_js_state());
    }

    fn stop() {
        clear(get_js_state());
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::pxs_Module>) {
        let state = get_js_state();
        unsafe {
            // let modules = state.modules.borrow();
            if (*state).modules.contains_key(&source.name) {
                // Don't add it
                pxs_debug!("JSModule {} already exists.", &source.name);
                return;
            }
        }
        module::add_module(get_context(state), &source);
    }

    fn execute(code: &str, file_name: &str) -> PxsResult {
        let res = run_js(code, file_name, quickjs::JS_EVAL_TYPE_MODULE as i32);
        let pxs_res = js_into_pxs(&res);
        if let Err(err) = pxs_res {
            Ok(pxs_Var::new_exception(err.to_string()))
        } else {
            let val = pxs_res.unwrap();
            if val.is_exception() {
                Ok(val)
            } else {
                Ok(pxs_Var::new_null())
            }
        }
    }

    fn eval(code: &str, name: &str) -> PxsResult {
        let res = run_js(code, name, quickjs::JS_EVAL_TYPE_GLOBAL as i32);
        js_into_pxs(&res)
    }

    fn start_thread() {
        // This is dangerous and could potentially break your program.
        // You must only call this in a real thread
        Self::start();
    }

    fn stop_thread() {
        // This is dangerous and could potentially break your program.
        // You must only call this in a real thread
        Self::stop();
    }

    fn clear() {
        let state = get_js_state();
        clear(state);
        init(state);
    }

    fn compile(code: &str, global_scope: crate::shared::var::pxs_Var) -> PxsResult {
        // Compile object
        let mod_obj = run_js(
            code,
            "<code_object>",
            (quickjs::JS_EVAL_FLAG_COMPILE_ONLY | quickjs::JS_EVAL_TYPE_MODULE) as i32,
        );
        if mod_obj.is_exception() || mod_obj.is_error() {
            return pxs_error!("{}", mod_obj.get_error_exception().unwrap());
        }

        // Execute it for the first time (there needs to be a specific function).
        let res = SmartJSValue::new_owned(
            unsafe { quickjs::JS_EvalFunction(mod_obj.context, mod_obj.value) },
            mod_obj.context,
        );
        // res.await_if_promise();

        if res.is_exception() || res.is_error() {
            return pxs_error!("{}", res.get_error_exception().unwrap());
        }

        // Save the `mod_obj` as pxs
        let pxs_val = js_into_pxs(&mod_obj)?;

        // Now lets convert global_scope into a JS object, then into a PXS object
        let global_scope_js_object = pxs_into_js(mod_obj.context, &global_scope)?;
        // Now back to pxs
        let global_scope_pxs = js_into_pxs(&global_scope_js_object)?;

        // Now lets return our [CodeObject, Global Scope reference]
        let result = pxs_Var::new_list();
        let list = result.get_list().unwrap();

        // Code object
        list.add_item(pxs_val);
        // Global object
        list.add_item(global_scope_pxs);

        Ok(result)
    }

    fn exec_object(
        code: crate::shared::var::pxs_Var,
        local_scope: crate::shared::var::pxs_Var,
    ) -> PxsResult {
        let state = get_js_state();
        let list = code.get_list().unwrap();

        let code_object_pxs = list.get_item(1).unwrap();
        let global_scope = list.get_item(2).unwrap();

        // Convert code object to JS
        let code_object_js = pxs_into_js(get_context(state), &code_object_pxs)?;
        if !code_object_js.is_module() {
            return pxs_error!("Expected module, found: {}", code_object_js.type_string());
        }

        // Get namespace and __pxs__ method
        let ns = code_object_js.get_module_namespace();
        let pxs_method = ns.get_prop(PXS_METHOD_NAME);
        if !pxs_method.is_function() {
            return pxs_error!(
                "Expected function for __pxs__, found: {}",
                pxs_method.type_string()
            );
        }

        let args = vec![
            pxs_into_js(get_context(state), &global_scope)?,
            pxs_into_js(get_context(state), &local_scope)?,
        ];

        // Call method
        let res = pxs_method.call_as_source(&args);

        if res.is_exception() {
            Ok(pxs_Var::new_exception(res.get_error_exception().unwrap()))
        } else {
            js_into_pxs(&res)
        }
    }

    fn debug() -> String {
        unsafe {
            let state = get_js_state();
            let binding = &(*state).defined_objects;
            let defined_objects = binding.keys();
            let binding = &(*state).module_exports;
            let module_exports = binding.keys();
            let binding = &(*state).modules;
            let module_names = binding.keys();

            format!(
                "{{defined_objects: {:#?}\nmodule_exports: {:#?}\nmodule_names:{:#?}}}",
                defined_objects, module_exports, module_names
            )
        }
    }

    fn garbage_collect() {
        let state = get_js_state();
        unsafe {
            if !(*state).rt.is_null() {
                quickjs::JS_RunGC((*state).rt);
            }
        }
    }
}

impl ObjectMethods for JSScripting {
    fn object_call(
        var: &crate::shared::var::pxs_Var,
        method: &str,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> PxsResult {
        let state = get_js_state();
        let js_var = pxs_into_js(get_context(state), var)?;
        let mut argv = vec![];

        for a in args.vars.iter() {
            argv.push(pxs_into_js(get_context(state), a)?);
        }

        let res = js_var.call(method, &argv);

        js_into_pxs(&res)
    }

    fn call_method(method: &str, args: &mut crate::shared::var::pxs_VarList) -> PxsResult {
        // globalThis.`method`(args)
        let state = get_js_state();
        let mut argv = vec![];
        for arg in args.vars.iter() {
            argv.push(pxs_into_js(get_context(state), arg)?);
        }

        // Look for method in global_this or eval
        let cbk = get_js_name(method);

        if !cbk.is_function() {
            // js_into_pxs(&cbk)
            Ok(pxs_Var::new_exception(format!(
                "{method} is not a Function"
            )))
        } else {
            let res = cbk.call_as_source(&argv);
            js_into_pxs(&res)
        }
    }

    fn var_call(
        method: &crate::shared::var::pxs_Var,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> PxsResult {
        let state = get_js_state();
        let smart_val = pxs_into_js(get_context(state), method)?;
        let mut argv = vec![];
        for arg in args.vars.iter() {
            argv.push(pxs_into_js(get_context(state), arg)?);
        }

        let res = smart_val.call_as_source(&argv);
        js_into_pxs(&res)
    }

    fn get(var: &crate::shared::var::pxs_Var, key: &str) -> PxsResult {
        let state = get_js_state();
        let this = pxs_into_js(get_context(state), var)?;
        let res = this.get_prop(key);

        js_into_pxs(&res)
    }

    fn set(
        var: &crate::shared::var::pxs_Var,
        key: &str,
        value: &crate::shared::var::pxs_Var,
    ) -> PxsRes<()> {
        let state = get_js_state();
        let this = pxs_into_js(get_context(state), var)?;
        let mut value = pxs_into_js(get_context(state), value)?;

        this.set_prop(key, &mut value);

        Ok(())
    }

    fn get_from_name(name: &str) -> PxsResult {
        js_into_pxs(&get_js_name(name))
    }
}
