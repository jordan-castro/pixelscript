use crate::{
    js::{
        SmartJSValue, quickjs,
        var::{js_into_pxs, pxs_into_js},
    }, pxs_debug, shared::{PXS_PTR_NAME, func::call_function, object::ObjectFlags, pxs_Runtime, var::pxs_Var}
};

/// Object callback trampoline
unsafe extern "C" fn object_trampoline(
    ctx: *mut quickjs::JSContext,
    this_val: quickjs::JSValue,
    argc: i32,
    argv: *mut quickjs::JSValue,
    _magic: i32,
    func_data: *mut quickjs::JSValue,
) -> quickjs::JSValue {
    // Convert JSValue -> vec![pxs_Var]
    let mut pxs_args = vec![
        pxs_Runtime::pxs_JavaScript.into_var()
    ];
    // Wrap in smart value
    let smart_this = SmartJSValue::new_borrow(this_val, ctx);

    // Has:
    // - fn idx
    // - flags
    unsafe {
        let flags = SmartJSValue::new_borrow(*func_data.offset(2), ctx).as_i32().unwrap() as u8;
        if flags & (ObjectFlags::UsesId as u8) != 0 {
            // Pass pxs_ptr 
            let pxs_ptr = smart_this.get_prop(PXS_PTR_NAME).as_i32().unwrap();
            // let pxs_ptr = SmartJSValue::new_borrow(*func_data.offset(1), ctx).as_i32().unwrap();
            pxs_args.push(pxs_Var::new_i64(pxs_ptr as i64));
        } else {
            // Reference
            let js_ref = js_into_pxs(&smart_this);
            if let Err(err) = js_ref {
                let message = format!("JS Ref could not be created: {err}");
                return SmartJSValue::new_exception(ctx, message, "CallbackRefError".to_string()).dupped_value();
            } else {
                // Add it
                pxs_args.push(js_ref.unwrap());
            }
        }
    }

    for i in 0..argc {
        let val = unsafe { argv.offset(i as isize) };
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
        SmartJSValue::new_borrow(*func_data.offset(0), ctx).as_i32().unwrap()
    };

    // Call function
    let res = unsafe{call_function(fn_idx, pxs_args)};

    let js_res = pxs_into_js(ctx, &res);
    if let Err(err) = js_res {
        let error_msg = format!("Error in callback: {}", err);
        pxs_debug!("{error_msg}");
        let res = SmartJSValue::new_exception(ctx, error_msg, "CallbackError".to_string());
        res.dupped_value()
    } else {
        let res = js_res.unwrap();
        res.dupped_value()
    }

}

/// Callback trampoline
unsafe extern "C" fn method_trampoline(
    ctx: *mut quickjs::JSContext,
    _this_val: quickjs::JSValue,
    argc: i32,
    argv: *mut quickjs::JSValue,
    _magic: i32,
    func_data: *mut quickjs::JSValue,
) -> quickjs::JSValue {
    // Convert JSValue -> vec![pxs_Var]
    let mut pxs_args = vec![
        pxs_Runtime::pxs_JavaScript.into_var()
    ];

    for i in 0..argc {
        let val = unsafe { argv.offset(i as isize) };
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
        SmartJSValue::new_borrow(*func_data.offset(0), ctx).as_i32().unwrap()
    };

    // Call function
    let res = unsafe{call_function(fn_idx, pxs_args)};

    let js_res = pxs_into_js(ctx, &res);
    if let Err(err) = js_res {
        let error_msg = format!("Error in callback: {}", err);
        pxs_debug!("{error_msg}");
        let res = SmartJSValue::new_exception(ctx, error_msg, "CallbackError".to_string());
        res.dupped_value()
    } else {
        let res = js_res.unwrap();
        res.dupped_value()
    }
}

/// Create a JS callback that gets attached to a module.
pub(super) fn create_callback(ctx: *mut quickjs::JSContext, fn_idx: i32) -> SmartJSValue {
    let mut idx_wrapper = SmartJSValue::new_i32(ctx, fn_idx);
    idx_wrapper.owned = false;
    let func_data = vec![
        idx_wrapper.value
    ];
    let func_data_ptr = func_data.into_raw_parts();
    let function = unsafe {
        quickjs::JS_NewCFunctionData(ctx, Some(method_trampoline), 0, 0, 1, func_data_ptr.0)
    };
    SmartJSValue::new_owned(function, ctx)
}

/// Create a JS callback that gets attached to a object.
pub(super) fn create_object_callback(ctx: *mut quickjs::JSContext, fn_idx: i32, flags: u8) -> SmartJSValue {
    let idx_wrapper = SmartJSValue::new_i32(ctx, fn_idx);
    let flags_wrapper = SmartJSValue::new_i32(ctx, flags as i32);
    let func_data = vec![
        idx_wrapper.dupped_value(),
        flags_wrapper.dupped_value()
    ];
    let func_data_ptr = func_data.into_raw_parts();
    let function = unsafe {
        quickjs::JS_NewCFunctionData(ctx, Some(object_trampoline), 0, 0, 2, func_data_ptr.0)
    };
    SmartJSValue::new_owned(function, ctx)
}