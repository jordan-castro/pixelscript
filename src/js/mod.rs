use std::{cell::RefCell, collections::HashMap, sync::Arc};

use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use rquickjs::{Context, Ctx, Error, IntoJs, Module, Runtime, loader::{Loader, Resolver}};

use crate::{js::{func::create_callback, var::pxs_into_js}, shared::{PixelScript, PtrMagic, module::pxs_Module, read_file, var::{ObjectMethods, pxs_Var}}};

mod var;
mod func;
mod object;
mod module;

/// JS specific State.
struct State {
    /// The JS runtime.
    rt: Runtime,
    /// The `__main__` context. Each state DOES NOT get it's own context.
    main_context: Context,
    /// The Hashmap of defined modules
    modules: RefCell<HashMap<String, Arc<pxs_Module>>>
}

thread_local! {
    static JSTATE: ReentrantMutex<State> = ReentrantMutex::new(init_state());
}

/// The pxs-rquickjs resolver. It simply allows any string through.
struct PassthroughResolver;

impl Resolver for PassthroughResolver {
    fn resolve<'js>(&mut self, _ctx: &Ctx<'js>, _base: &str, name: &str) -> rquickjs::Result<String> {
        // We ignore 'base' entirely. 
        // Whatever they typed is the unique ID for the module cache.
        Ok(name.to_string())
    }
}

/// The pxs-rquickjs loader. It will try to load pxs modules first, then file paths.
struct JSModuleLoader {
}
impl JSModuleLoader {
    /// Setup a module in the loader. Should only be called once at instance.
    pub fn setup_new_module<'js>(ctx: &Ctx<'js>, module: &Arc<pxs_Module>) -> rquickjs::Result<rquickjs::Module<'js>> {
        let mut source_code = String::new();

        for var in module.variables.iter() {
            source_code.push_str(format!("export let {} = null;\n", var.name).as_str());
        }

        for func in module.callbacks.iter() {
            source_code.push_str(format!("export let {} = null;\n", func.name).as_str());
        }

        let m_def = Module::declare(ctx.clone(), module.name.clone(), source_code)?;
        let m = m_def.eval()?.0;
        
        let meta_obj = m.meta()?;

        for var in module.variables.iter() {
            let safe_var = unsafe {pxs_Var::from_borrow(var.var) };
            meta_obj.set(var.name.clone(), pxs_into_js(&ctx, safe_var)?)?;
        }

        for func in module.callbacks.iter() {
            let cbk = create_callback(ctx.clone(), func.idx);
            if cbk.is_err() {
                return Err(ctx.throw(cbk.unwrap_err().to_string().into_js(ctx)?));
            }
            meta_obj.set(func.name.clone(), cbk.unwrap())?;
        }

        Ok(m.into_declared())
    }
}
impl Loader for JSModuleLoader {
    fn load<'js>(&mut self, ctx: &rquickjs::Ctx<'js>, name: &str) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        // Check if name exists 
        let state = get_js_state();
        let modules = state.modules.borrow();
        if modules.contains_key(name) {
            // We have a module
            return JSModuleLoader::setup_new_module(ctx, modules.get(name).unwrap());
        }

        // Check file path
        let contents = read_file(name);
        if !contents.is_empty() {
            let m = Module::declare(ctx.clone(), name, contents)?;
            return Ok(m.eval()?.0.into_declared());
        }

        Err(Error::new_loading(name))
    }
}

/// Initialize the JS state.
fn init_state() -> State {
    let runtime = Runtime::new().expect("JS Runtime could not be created.");
    let context = Context::full(&runtime).expect("JS Context could not be created.");

    let resolver = PassthroughResolver{};
    let loader = JSModuleLoader{};

    // Module loader
    runtime.set_loader((resolver,), (loader,));

    // TODO: setup pxs stuff.

    State { rt: runtime, main_context: context, modules: RefCell::new(HashMap::new()) }
}

fn get_js_state() -> ReentrantMutexGuard<'static, State> {
    JSTATE.with(|mutex| {
        let guard = mutex.lock();
        // Transmute the lifetime so the guard can be passed around the thread
        unsafe { std::mem::transmute(guard) }
    })
}

pub struct JSScripting;

impl PixelScript for JSScripting {
    fn start() {
        init_state();
    }

    fn stop() {
        let state = get_js_state();
        state.rt.run_gc();
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::pxs_Module>) {
        let _ = module::add_module(source);
    }

    fn execute(code: &str, file_name: &str) -> anyhow::Result<crate::shared::var::pxs_Var> {
        todo!()
    }

    fn eval(code: &str) -> anyhow::Result<crate::shared::var::pxs_Var> {
        todo!()
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

    fn compile(code: &str, global_scope: crate::shared::var::pxs_Var) -> anyhow::Result<crate::shared::var::pxs_Var> {
        todo!()
    }

    fn exec_object(code: crate::shared::var::pxs_Var, local_scope: crate::shared::var::pxs_Var) -> anyhow::Result<crate::shared::var::pxs_Var> {
        todo!()
    }
}

impl ObjectMethods for JSScripting {
    fn object_call(var: &crate::shared::var::pxs_Var, method: &str, args: &mut crate::shared::var::pxs_VarList) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }

    fn call_method(method: &str, args: &mut crate::shared::var::pxs_VarList) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }

    fn var_call(method: &crate::shared::var::pxs_Var, args: &mut crate::shared::var::pxs_VarList) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }

    fn get(var: &crate::shared::var::pxs_Var, key: &str) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }

    fn set(var: &crate::shared::var::pxs_Var, key: &str, value: &crate::shared::var::pxs_Var) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }

    fn get_from_name(name: &str) -> Result<crate::shared::var::pxs_Var, anyhow::Error> {
        todo!()
    }
}