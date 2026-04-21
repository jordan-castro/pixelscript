use anyhow::{Result, anyhow};

use crate::{
    js::{
        SmartJSValue, quickjs,
        var::{js_into_pxs, pxs_into_js},
    }, pxs_debug, shared::{func::call_function, pxs_Runtime}
};

/// Callback trampoline
fn create_trampoline(
    ctx: *mut quickjs::JSContext,
    this_val: quickjs::JSValue,
    argc: i32,
    argv: *mut quickjs::JSValue,
    func_data: *mut quickjs::JSValue,
) -> quickjs::JSValue {
    // Convert JSValue -> vec![pxs_Var]
    let mut pxs_args = vec![
        pxs_Runtime::pxs_JavaScript.into_var()
    ];
    for i in 0..argc {
        let val = unsafe { argv.add(i as usize) };
        let smart_val = SmartJSValue::new_borrow(unsafe{ *val }, ctx);
        let pxs_val = js_into_pxs(&smart_val);
        if pxs_val.is_err() {
            pxs_debug!("Error in callback: {:#?}", pxs_val.unwrap_err());
            continue;
        }
        pxs_args.push(pxs_val.unwrap());
    }

    // Get function idx from magic
    let fn_idx = unsafe{
        SmartJSValue::new_borrow(*func_data.add(0), ctx).as_i32().unwrap()
    };

    // Call function
    let res = unsafe{call_function(fn_idx, pxs_args)};

    let js_res = pxs_into_js(ctx, &res);
    if js_res.is_err() {
        pxs_debug!("Error in callback: {:#?}", js_res.unwrap_err());
        let res = SmartJSValue::new_exception(ctx, format!("Error in callback: {:#?}", js_res.unwrap_err()), "CallbackError".to_string());
        res.owned = false;
        res.value
    } else {
        let res = js_res.unwrap();
        res.owned = false;
        res.value
    }
}

/// Create a JS callback. Can be assigned to a module, object, etc.
pub(super) fn create_callback(ctx: *mut quickjs::JSContext, fn_idx: i32) -> Result<SmartJSValue> {
    // let func = Function::new(ctx.clone(), move |ctx: Ctx<'js>, args: Rest<Value>| -> rquickjs::Result<Value> {
    //     // Convert args -> pxs args
    //     let mut argv = vec![];

    //     // Pass in runtime
    //     argv.push(pxs_Runtime::pxs_JavaScript.into_var());

    //     for arg in args.0 {
    //         let js_arg = js_into_pxs(arg);
    //         if js_arg.is_err() {
    //             return Err(ctx.throw(js_arg.unwrap_err().to_string().into_js(&ctx)?))
    //         }
    //         argv.push(js_arg.unwrap());
    //     }

    //     // Call pxs function
    //     unsafe {
    //         let res = call_function(fn_idx, argv);
    //         let js_val = pxs_into_js(&ctx, &res);
    //         js_val
    //     }
    // })?;

    Ok(func)
}
