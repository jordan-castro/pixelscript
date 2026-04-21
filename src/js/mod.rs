use std::{cell::RefCell, collections::HashMap, sync::Arc};

use anyhow::{Result, anyhow};
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};

use crate::{
    js::{
        func::create_callback, utils::{SmartJSValue, is_int, is_undefined}, var::{js_into_pxs, pxs_into_js}
    },
    pxs_debug,
    shared::{
        PixelScript, PtrMagic,
        module::pxs_Module,
        read_file,
        utils::CStringSafe,
        var::{ObjectMethods, pxs_Var},
    },
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

/// JS specific State.
struct State {
    /// The JS runtime.
    rt: *mut quickjs::JSRuntime,
    /// The `__main__` context. Each state DOES NOT get it's own context.
    context: *mut quickjs::JSContext,
}

thread_local! {
    static JSTATE: ReentrantMutex<State> = ReentrantMutex::new(unsafe{init_state()});
}

// /// The pxs-rquickjs resolver. It simply allows any string through.
// struct PassthroughResolver;

// impl Resolver for PassthroughResolver {
//     fn resolve<'js>(&mut self, _ctx: &Ctx<'js>, _base: &str, name: &str) -> rquickjs::Result<String> {
//         pxs_debug!("{name}");
//         // We ignore 'base' entirely.
//         // Whatever they typed is the unique ID for the module cache.
//         Ok(name.to_string())
//     }
// }

// /// The pxs-rquickjs loader. It will try to load pxs modules first, then file paths.
// struct JSModuleLoader {
// }
// impl JSModuleLoader {
//     /// Setup a module in the loader. Should only be called once at instance.
//     pub fn setup_new_module<'js>(ctx: &Ctx<'js>, module: &Arc<pxs_Module>) -> rquickjs::Result<rquickjs::Module<'js>> {
//         let mut source_code = String::new();

//         for var in module.variables.iter() {
//             source_code.push_str(format!("export let {} = null;\n", var.name).as_str());
//         }

//         for func in module.callbacks.iter() {
//             source_code.push_str(format!("export let {} = null;\n", func.name).as_str());
//         }

//         let m_def = Module::declare(ctx.clone(), module.name.clone(), source_code)?;
//         let m = m_def.eval()?.0;

//         let meta_obj = m.meta()?;

//         for var in module.variables.iter() {
//             let safe_var = unsafe {pxs_Var::from_borrow(var.var) };
//             meta_obj.set(var.name.clone(), pxs_into_js(&ctx, safe_var)?)?;
//         }

//         for func in module.callbacks.iter() {
//             let cbk = create_callback(ctx.clone(), func.idx);
//             if cbk.is_err() {
//                 return Err(ctx.throw(cbk.unwrap_err().to_string().into_js(ctx)?));
//             }
//             meta_obj.set(func.name.clone(), cbk.unwrap())?;
//         }

//         Ok(m.into_declared())
//     }
// }
// impl Loader for JSModuleLoader {
//     fn load<'js>(&mut self, ctx: &rquickjs::Ctx<'js>, name: &str) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
//         // Check if name exists
//         let state = get_js_state();
//         let modules = state.modules.borrow();
//         if modules.contains_key(name) {
//             // We have a module
//             return JSModuleLoader::setup_new_module(ctx, modules.get(name).unwrap());
//         }

//         // Check file path
//         let contents = read_file(name);
//         if !contents.is_empty() {
//             let m = Module::declare(ctx.clone(), name, contents)?;
//             return Ok(m.eval()?.0.into_declared());
//         }

//         Err(Error::new_loading(name))
//     }
// }

/// Initialize the JS state.
unsafe fn init_state() -> State {
    unsafe {
        let rt = quickjs::JS_NewRuntime();
        let ctx = quickjs::JS_NewContext(rt);

        // TODO: load main.js
        // TODO: setup pxs_json

        State { rt, context: ctx }
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

pub struct JSScripting;

impl PixelScript for JSScripting {
    fn start() {
        let _state = get_js_state();
    }

    fn stop() {
        let state = get_js_state();
        unsafe {
            quickjs::JS_RunGC(state.rt);
        }
        // state.rt.run_gc();
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::pxs_Module>) {
        let _ = module::add_module(source);
    }

    fn execute(code: &str, file_name: &str) -> anyhow::Result<crate::shared::var::pxs_Var> {
        let state = get_js_state();
        let res = state.main_context.with(|ctx| -> anyhow::Result<pxs_Var> {
            let promise = Module::evaluate(ctx.clone(), file_name, code)?;
            let val: Value = promise.finish()?;
            let pxs = js_into_pxs(val);
            if pxs.is_err() {
                Err(anyhow!("{}", pxs.unwrap_err().to_string()))
            } else {
                Ok(pxs.unwrap())
            }
        })?;
        if res.is_exception() {
            Ok(res)
        } else {
            Ok(pxs_Var::new_null())
        }
    }

    fn eval(code: &str) -> anyhow::Result<crate::shared::var::pxs_Var> {
        let state = get_js_state();
        let res = state.main_context.with(|ctx| -> anyhow::Result<pxs_Var> {
            let promise = ctx.eval_promise(code)?;
            let val: Value = promise.finish()?;
            let pxs = js_into_pxs(val);
            if pxs.is_err() {
                Err(anyhow!("{}", pxs.unwrap_err().to_string()))
            } else {
                Ok(pxs.unwrap())
            }
        })?;
        Ok(res)
    }

    fn start_thread() {
        // Not needed for JS.
    }

    fn stop_thread() {
        // Not needed for JS.
    }

    fn clear_state(call_gc: bool) {
        let state = get_js_state();

        if call_gc {
            state.rt.run_gc();
        }
    }

    fn compile(
        code: &str,
        global_scope: crate::shared::var::pxs_Var,
    ) -> anyhow::Result<crate::shared::var::pxs_Var> {
        todo!()
    }

    fn exec_object(
        code: crate::shared::var::pxs_Var,
        local_scope: crate::shared::var::pxs_Var,
    ) -> anyhow::Result<crate::shared::var::pxs_Var> {
        todo!()
    }
}

impl ObjectMethods for JSScripting {
    fn object_call(
        var: &crate::shared::var::pxs_Var,
        method: &str,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }

    fn call_method(
        method: &str,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }

    fn var_call(
        method: &crate::shared::var::pxs_Var,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }

    fn get(
        var: &crate::shared::var::pxs_Var,
        key: &str,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }

    fn set(
        var: &crate::shared::var::pxs_Var,
        key: &str,
        value: &crate::shared::var::pxs_Var,
    ) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }

    fn get_from_name(name: &str) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }
}
