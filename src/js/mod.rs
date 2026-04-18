use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use rquickjs::{Context, Runtime};

use crate::shared::{PixelScript, var::ObjectMethods};

mod var;
mod func;

/// JS specific State.
struct State {
    /// The JS runtime.
    rt: Runtime,
    /// The `__main__` context. Each state DOES NOT get it's own context.
    main_context: Context,
}

thread_local! {
    static JSTATE: ReentrantMutex<State> = ReentrantMutex::new(init_state());
}

/// Initialize the JS state.
fn init_state() -> State {
    let runtime = Runtime::new().expect("JS Runtime could not be created.");
    let context = Context::full(&runtime).expect("JS Context could not be created.");
    
    // TODO: setup pxs stuff.

    State { rt: runtime, main_context: context }
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
        todo!()
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