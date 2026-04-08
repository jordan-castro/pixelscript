use parking_lot::ReentrantMutex;
use rquickjs::{Context, Runtime};

use crate::shared::{PixelScript, var::ObjectMethods};

/// JS specific State.
struct State {
    /// The JS runtime.
    rt: Runtime,
    /// The `__main__` context. Each state DOES NOT get it's own context.
    main_context: Context
}

thread_local! {
    static JSTATE: ReentrantMutex<State> = ReentrantMutex::new(init_state());
}

/// Initialize the JS state.
fn init_state() -> State {
    let runtime = Runtime::new().expect("Could not load JS Runtime");
    // let ctx = runtime.

    State { rt: runtime }
//     /// Initialize Lua state per thread.
// fn init_state() -> State {
//     // Define a global function in engine
//     let engine = Lua::new();

//     let mut lua_globals = String::new();
//     lua_globals.push_str(include_str!("../../core/lua/main.lua"));

//     // with_feature!("pxs_utils", {
//     //     // Load in the pxs_utils methods into GLOBAL scope.
//     //     lua_globals.push_str(include_str!("../../core/lua/pxs_utils.lua"));
//     // });

//     with_feature!("pxs_json", {
//         // Load dkjson module
//         let _ = preload_lua_module(&engine, include_str!("../../libs/dkjson.lua"), "__dkjson__");
//         // Load in the pxs_json module
//         let _ = preload_lua_module(&engine, include_str!("../../core/lua/pxs_json.lua"), "pxs_json");
//         // Import it globally
//         lua_globals.push_str("\npxs_json = require('pxs_json')\n");
//     });
//     let _ = engine.load(lua_globals).set_name("<lua_globals>").exec();

//     State {
//         engine: engine,
//         tables: RefCell::new(HashMap::new()),
//     }
// }

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