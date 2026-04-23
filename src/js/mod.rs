use std::{cell::RefCell, collections::HashMap};

use anyhow::{Result, anyhow};
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};

use crate::{
    borrow_string, js::{
        func::create_callback, utils::SmartJSValue, var::{js_into_pxs, pxs_into_js}
    }, pxs_debug, shared::{
        PixelScript, pxs_Opaque, read_file, utils::CStringSafe, var::{ObjectMethods, pxs_Var, pxs_VarMap}
    }
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
mod var;
mod utils;

/// Holds name and Value for module methods
pub(self) struct JSModuleMethod {
    pub name: String,
    pub value: SmartJSValue
}

/// JS specific State.
struct State {
    /// The JS runtime.
    rt: *mut quickjs::JSRuntime,
    /// The `__main__` context. Each state DOES NOT get it's own context.
    context: *mut quickjs::JSContext,
    /// Keep a list of defined PixelObject as class
    defined_objects: RefCell<HashMap<String, SmartJSValue>>,
    /// Module defined functions takes a map[module_name] => map[int] => function
    module_functions: RefCell<HashMap<String, Vec<JSModuleMethod>>>,
    /// JSModules
    modules: RefCell<HashMap<String, *mut quickjs::JSModuleDef>>,
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe {
            if self.rt != std::ptr::null_mut() {
                quickjs::JS_FreeRuntime(self.rt);
            }
            if self.context != std::ptr::null_mut() {
                quickjs::JS_FreeContext(self.context);
            }
        }
    }
}

thread_local! {
    static JSTATE: ReentrantMutex<State> = ReentrantMutex::new(unsafe{init_state()});
}

/// JS Module loader
unsafe extern "C" fn js_module_loader(context: *mut quickjs::JSContext, module_name: *const std::ffi::c_char, _opaque: pxs_Opaque) -> *mut quickjs::JSModuleDef {
    let state = get_js_state();
    let modules = state.modules.borrow();
    unsafe {
        let name = borrow_string!(module_name);
        if let Some(module) = modules.get(name) {
            return *module;
        }

        // Otherwise try to read the file...
        let contents = read_file(name);
        if contents.len() == 0 {
            return std::ptr::null_mut();
        }

        let mut cstrsafe = CStringSafe::new();
        // We need to evalute a module
        let res = quickjs::JS_Eval(context, cstrsafe.new_string(&contents), contents.len(), module_name, (quickjs::JS_EVAL_TYPE_MODULE | quickjs::JS_EVAL_FLAG_COMPILE_ONLY) as i32);
        let smart_res = SmartJSValue::new_owned(res, context);

        // Check exception
        if smart_res.is_exception() {
            pxs_debug!("Error compiling module");
            return std::ptr::null_mut();
        }

        let val_int = smart_res.value.u.ptr as isize;
        let m = ((val_int & !15) as *mut std::ffi::c_void).cast::<quickjs::JSModuleDef>();

        m
    }
}

/// Initialize the JS state.
unsafe fn init_state() -> State {
    unsafe {
        let rt = quickjs::JS_NewRuntime();
        let ctx = quickjs::JS_NewContext(rt);

        // load main.js
        let mut js_globals = String::new();
        js_globals.push_str(include_str!("../../core/js/main.js"));
        // TODO: setup pxs_json

        let mut cstrsafe = CStringSafe::new();
        quickjs::JS_Eval(ctx, cstrsafe.new_string(&js_globals), js_globals.len(), cstrsafe.new_string("<js_globals>"), quickjs::JS_EVAL_TYPE_GLOBAL as i32);

        // Setup module loader!
        quickjs::JS_SetModuleLoaderFunc(rt, None, Some(js_module_loader), std::ptr::null_mut());

        State { rt, context: ctx, defined_objects: RefCell::new(HashMap::new()), module_functions: RefCell::new(HashMap::new()), modules: RefCell::new(HashMap::new()) }
    }
}

fn get_js_state() -> ReentrantMutexGuard<'static, State> {
    JSTATE.with(|mutex| {
        let guard = mutex.lock();
        // Transmute the lifetime so the guard can be passed around the thread
        unsafe { std::mem::transmute(guard) }
    })
}

/// Add a new object to our PXS_Register
pub(self) fn register_add_object(value: SmartJSValue) -> Result<i32> {
    // Get globalThis.PXS_Register and call new_register(value)
    let state = get_js_state();
    let global_this = SmartJSValue::globalThis(state.context);
    let pxs_register = global_this.get_prop("PXS_Register");
    
    // Undefined check
    if pxs_register.is_undefined() {
        return Err(anyhow!("pxs_register is not defined."));
    }

    let argv = vec![value];

    // Call new_register
    let result = pxs_register.call("new_register", &argv);

    // Who knows.
    if !result.is_int() {
        Err(anyhow!("Could not register value."))
    } else {
        result.as_i32()
    }
}

/// Get a object from our PXS_Register
pub(self) fn register_get_object(idx: i32) -> SmartJSValue {
    let state = get_js_state();
    let global_this = SmartJSValue::globalThis(state.context);
    let pxs_register = global_this.get_prop("PXS_Register");

    if pxs_register.is_undefined() {
        return SmartJSValue::new_undefined(state.context);
    }

    let objects = pxs_register.get_prop("objects");
    objects.get_prop(idx.to_string())
}

/// Remove a object from our PXS_Register
pub(self) fn register_del_object(idx: i32) {
    let state = get_js_state();
    let global_this = SmartJSValue::globalThis(state.context);
    let pxs_register = global_this.get_prop("PXS_Register");

    if pxs_register.is_undefined() {
        return;
    }

    let objects = pxs_register.get_prop("objects");
    let prop = SmartJSValue::new_i32(state.context, idx);
    objects.del_prop(&prop);
}

/// Run JS code
fn run_js(code: &str, file_name: &str, eval_type: i32) -> SmartJSValue {
    let mut cstrsafe = CStringSafe::new();
    let state = get_js_state();
    unsafe {
        let val = quickjs::JS_Eval(state.context, cstrsafe.new_string(code), code.len(), cstrsafe.new_string(file_name), eval_type);

        // Check for exception
        // let exception = SmartJSValue::current_exception(state.context);
        // if exception.is_undefined() {
            SmartJSValue::new_owned(val, state.context)        
        // } else {
        //     exception
        // }
    }
}

/// Add pxs_Map to globalThis
fn add_map_to_global_this(map: &pxs_VarMap, global_this: &SmartJSValue) -> Result<()> {
    for key in map.keys() {
        let js_key = pxs_into_js(global_this.context, key)?;
        let js_val = pxs_into_js(global_this.context, map.get_item(key).unwrap())?;
        global_this.set_prop_value(&js_key, &js_val);
    }

    Ok(())
}

/// Remove pxs_Map from globalThis
fn remove_map_from_global_this(map: &pxs_VarMap, global_this: &SmartJSValue) -> Result<()> {
    for key in map.keys() {
        let js_key = pxs_into_js(global_this.context, key)?;
        global_this.del_prop(&js_key);
    }

    Ok(())
}

pub struct JSScripting;

impl PixelScript for JSScripting {
    fn start() {
        let _state = get_js_state();
    }

    fn stop() {
        Self::clear_state(true);
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::pxs_Module>) {
        let state = get_js_state();
        module::add_module(state.context, source);
    }

    fn execute(code: &str, file_name: &str) -> anyhow::Result<crate::shared::var::pxs_Var> {
        let res = run_js(code, file_name, quickjs::JS_EVAL_TYPE_MODULE as i32);
        let pxs_res = js_into_pxs(&res);
        if let Err(err) = pxs_res {
            Ok(pxs_Var::new_exception(err.to_string()))
        } else {
            Ok(pxs_Var::new_null())
        }
    }

    fn eval(code: &str) -> anyhow::Result<crate::shared::var::pxs_Var> {
        let res = run_js(code, "<eval>", quickjs::JS_EVAL_TYPE_GLOBAL as i32);
        js_into_pxs(&res)
    }

    fn start_thread() {
        // Not needed for JS.
    }

    fn stop_thread() {
        // Not needed for JS.
    }

    fn clear_state(call_gc: bool) {
        let state = get_js_state();
        // Clear state stuff first.
        state.defined_objects.borrow_mut().clear();
        state.module_functions.borrow_mut().clear();
        state.modules.borrow_mut().clear();
        if call_gc {
            unsafe {
                quickjs::JS_RunGC(state.rt);
            }
        }
    }

    fn compile(
        code: &str,
        global_scope: crate::shared::var::pxs_Var,
    ) -> anyhow::Result<crate::shared::var::pxs_Var> {
        // Compile object
        let obj = run_js(code, "<code_object>", (quickjs::JS_EVAL_FLAG_COMPILE_ONLY | quickjs::JS_EVAL_TYPE_MODULE) as i32);
        // // Now lets return our [CodeObject, Global Scope reference]
        let result = pxs_Var::new_list();
        let list = result.get_list().unwrap();

        // Code object
        list.add_item(js_into_pxs(&obj)?);
        // Global object
        list.add_item(global_scope);

        Ok(result)
    }

    fn exec_object(
        code: crate::shared::var::pxs_Var,
        local_scope: crate::shared::var::pxs_Var,
    ) -> anyhow::Result<crate::shared::var::pxs_Var> {
        let state = get_js_state();
        let list = code.get_list().unwrap();
        let obj = pxs_into_js(state.context, list.get_item(1).unwrap())?;
        // Add globalThis
        let global_this = SmartJSValue::globalThis(state.context);
        let global_scope = list.get_item(2).unwrap();

        if !global_scope.is_null() {
            add_map_to_global_this(global_scope.get_map().unwrap(), &global_this)?;
        }

        if !local_scope.is_null() {
            add_map_to_global_this(local_scope.get_map().unwrap(), &global_this)?;
        }

        // Execute this dude
        let res = SmartJSValue::new_owned(unsafe {
            quickjs::JS_EvalFunction(state.context, obj.dupped_value())
        }, state.context);

        // Remove from globalThis
        remove_map_from_global_this(global_scope.get_map().unwrap(), &global_this)?;
        if !local_scope.is_null() {
            remove_map_from_global_this(local_scope.get_map().unwrap(), &global_this)?;
        }
        if res.is_exception() {
            Ok(pxs_Var::new_exception(res.get_error_exception().unwrap()))
        } else {
            js_into_pxs(&res)
        }
    }
}

impl ObjectMethods for JSScripting {
    fn object_call(
        var: &crate::shared::var::pxs_Var,
        method: &str,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        let state = get_js_state();
        let js_var = pxs_into_js(state.context, var)?;
        let mut argv = vec![];

        for a in args.vars.iter() {
            argv.push(pxs_into_js(state.context, a)?);
        }

        let res = js_var.call(method, &argv);

        js_into_pxs(&res)
    }

    fn call_method(
        method: &str,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        // globalThis.`method`(args)
        let state = get_js_state();
        let global_this = SmartJSValue::globalThis(state.context);
        let mut argv = vec![];
        for arg in args.vars.iter() {
            argv.push(pxs_into_js(state.context, arg)?);
        }

        // Look for method in global_this or eval
        let cbk = global_this.get_prop(method);

        if !cbk.is_function() {
            // js_into_pxs(&cbk)
            Ok(pxs_Var::new_exception(format!("{method} is not a Function")))
        } else {
            let res = cbk.call_as_source(&argv);
            js_into_pxs(&res)
        }
    }

    fn var_call(
        method: &crate::shared::var::pxs_Var,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        let state = get_js_state();
        let smart_val = pxs_into_js(state.context, method)?;
        let mut argv = vec![];
        for arg in args.vars.iter() {
            argv.push(pxs_into_js(state.context, arg)?);
        }

        let res = smart_val.call_as_source(&argv);
        js_into_pxs(&res)
    }

    fn get(
        var: &crate::shared::var::pxs_Var,
        key: &str,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        let state = get_js_state();
        let this = pxs_into_js(state.context, var)?;
        let res = this.get_prop(key);
        js_into_pxs(&res)
    }

    fn set(
        var: &crate::shared::var::pxs_Var,
        key: &str,
        value: &crate::shared::var::pxs_Var,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        let state = get_js_state();
        let this = pxs_into_js(state.context, var)?;
        let value = pxs_into_js(state.context, value)?;
        
        this.set_prop(key, &value);

        Ok(pxs_Var::new_bool(true))
    }

    fn get_from_name(name: &str) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        let state = get_js_state();
        let global_this = SmartJSValue::globalThis(state.context);
        let res = {
            let pos = global_this.get_prop(name);
            if !pos.is_undefined() {pos} else {
                run_js(name, "<get_from_name_eval>", quickjs::JS_EVAL_TYPE_GLOBAL as i32)
            }
        };
        js_into_pxs(&res)
    }
}
