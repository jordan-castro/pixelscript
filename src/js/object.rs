use std::sync::Arc;

use crate::{js::{SmartJSValue, func::{create_object_callback}, get_js_state, quickjs}, shared::{PXS_PTR_NAME, object::pxs_PixelObject}};

unsafe extern "C" fn js_class_constructor(
    ctx: *mut quickjs::JSContext,
    this_val: quickjs::JSValue,
    _argc: i32,
    _argv: *mut quickjs::JSValue,
    _magic: i32,
    func_data: *mut quickjs::JSValue,
) -> quickjs::JSValue {
    // Get the pxs_ptr
    let pxs_ptr = SmartJSValue::new_borrow(unsafe{*func_data.offset(0)}, ctx);
    let smart_this = SmartJSValue::new_borrow(this_val, ctx);
    let proto = smart_this.get_prop("prototype");
    let mut obj = proto.new_object_proto();
    obj.dont_drop();
    obj.set_prop(PXS_PTR_NAME, &pxs_ptr);

    obj.value
}

pub(super) fn create_object(ctx: *mut quickjs::JSContext, idx: i32, source: Arc<pxs_PixelObject>) -> SmartJSValue {
    let state = get_js_state();
    let type_name = &source.type_name;

    let object = SmartJSValue::new_object(ctx);

    // Check if already exist
    let mut defined_objects = state.defined_objects.borrow_mut();
    if defined_objects.contains_key(type_name) {
        let dobj = defined_objects.get(type_name).unwrap();
        object.set_prop(PXS_PTR_NAME, &SmartJSValue::new_i32(ctx, idx));
        // Set proto
        object.set_proto(dobj);
        // DONE
        return object;
    }

    // Create new object
    for object_cbk in source.callbacks.iter() {
        let module_cbk = &object_cbk.cbk;
        let flags = object_cbk.flags;

        let func = create_object_callback(ctx, module_cbk.idx, flags);
        // Set
        object.set_prop(&module_cbk.name, &func);
    }

    // Define obj
    defined_objects.insert(type_name.clone(), object);

    // Recursion!
    create_object(ctx, idx, source)
}
