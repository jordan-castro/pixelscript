use rquickjs::Runtime;

use crate::shared::{PixelScript, var::ObjectMethods};

/// JS specific State.
struct State {
    rt: Runtime,
}

pub struct JSScripting;

impl PixelScript for JSScripting {
    fn start() {
        todo!()
    }

    fn stop() {
        todo!()
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
        todo!()
    }

    fn stop_thread() {
        todo!()
    }

    fn clear_state(call_gc: bool) {
        todo!()
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