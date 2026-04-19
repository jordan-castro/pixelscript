use anyhow::{Result, anyhow};
use rquickjs::{Ctx, Function, IntoJs, Value, prelude::Rest};

use crate::{js::var::{js_into_pxs, pxs_into_js}, shared::{func::call_function, pxs_Runtime, var::pxs_Var}};

/// Create a JS callback. Can be assigned to a module, object, etc.
pub(super) fn create_callback<'js>(ctx: Ctx<'js>, fn_idx: i32) -> Result<Function<'js>> {
    let func = Function::new(ctx.clone(), move |ctx: Ctx<'js>, args: Rest<Value>| -> rquickjs::Result<Value> {
        // Convert args -> pxs args
        let mut argv = vec![];

        // Pass in runtime
        argv.push(pxs_Runtime::pxs_JavaScript.into_var());

        for arg in args.0 {
            let js_arg = js_into_pxs(arg);
            if js_arg.is_err() {
                return Err(ctx.throw(js_arg.unwrap_err().to_string().into_js(&ctx)?))
            }
            argv.push(js_arg.unwrap());
        }

        // Call pxs function
        unsafe {
            let res = call_function(fn_idx, argv);
            let js_val = pxs_into_js(&ctx, &res);
            js_val
        }
    })?;

    Ok(func)
}