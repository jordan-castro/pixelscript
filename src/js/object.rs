use std::sync::Arc;

use rquickjs::{Ctx, Function, IntoJs, Object, Value, prelude::Rest};

use crate::{js::{func::create_callback, var::{js_into_pxs, pxs_into_js}}, shared::{func::call_function, object::{ObjectFlags, pxs_PixelObject}, pxs_Runtime, var::pxs_Var}};

fn create_object_callback<'js>(ctx: Ctx<'js>, fn_idx: i32, flags: u8) -> rquickjs::Result<Function<'js>> {
    let func = Function::new(ctx.clone(), move |this: Object, args: Rest<Value>| -> rquickjs::Result<Value> {
        let mut argv = vec![];
        // Add runtime
        argv.push(pxs_Runtime::pxs_JavaScript.into_var());

        // Check how to pass in the object ref
        if flags & (ObjectFlags::UsesId as u8) != 0 {
            let obj_id: i64 = this.get("_pxs_ptr")?;
            argv.push(pxs_Var::new_i64(obj_id));
        } else {
            // ref
            let obj_value = js_into_pxs(this.into_value());
            if obj_value.is_err() {
                return Err(ctx.throw(obj_value.unwrap_err().to_string().into_js(&ctx)?));
            }
            argv.push(obj_value.unwrap());
        }

        // Add arg
        for arg in args.0 {
            let js_arg = js_into_pxs(arg);
            if js_arg.is_err() {
                return Err(ctx.throw(js_arg.unwrap_err().to_string().into_js(&ctx)?));
            }
            argv.push(js_arg.unwrap());
        }

        // Call
        unsafe {
            let res = call_function(fn_idx, argv);
            let js_val = pxs_into_js(&ctx, &res);
            js_val
        }
    });

    func
    // if func.is_err() {
    //     Err(anyhow!("{:#?}", func.unwrap_err()))
    // } else {
    //     Ok(func.unwrap())
    // }
}

pub(super) fn create_object<'js>(ctx: &Ctx<'js>, idx: i32, source: Arc<pxs_PixelObject>) -> rquickjs::Result<Value<'js>> {
    let type_name = &source.type_name;

    // Constructor
    let constructor = Function::new(ctx.clone(), move |ctx:Ctx<'js>, ptr: i32| -> rquickjs::Result<Object> {
        let this = Object::new(ctx)?;
        this.set("_pxs_ptr", ptr)?;
        Ok(this)
    })?;

    // Prototype
    let proto: Object = constructor.get_prototype().unwrap();

    // Get `Object.defineProperty` method
    let js_object_ctor: Object = ctx.globals().get("Object")?;
    let define_property: Function = js_object_ctor.get("defineProperty")?;

    // Callbacks
    for method in source.callbacks.iter() {
        let cbk = &method.cbk;
        // Create callback.
        let func = create_object_callback(ctx.clone(), cbk.idx, method.flags)?;

        // Check property.
        if method.flags & ObjectFlags::IsProp as u8 != 0 {
            // Set as a Property
            let descriptor = Object::new(ctx.clone())?;
            descriptor.set("get", func.clone())?;
            descriptor.set("set", func.clone())?;

            let _: Object = define_property.call((proto.clone(), &cbk.name, descriptor))?;
        } else {
            // Just set (name, func)
            proto.set(&cbk.name, func)?;
        }
    }

    let new_obj = constructor.call((idx,))?;
    Ok(new_obj)
}
