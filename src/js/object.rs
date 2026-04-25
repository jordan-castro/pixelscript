use std::sync::Arc;

use crate::{js::{SmartJSValue, func::create_object_callback, get_js_state, quickjs}, shared::{PXS_PTR_NAME, object::pxs_PixelObject}};

pub(super) fn create_object(ctx: *mut quickjs::JSContext, idx: i32, source: Arc<pxs_PixelObject>) -> SmartJSValue {
    let state = get_js_state();
    let type_name = &source.type_name;

    let object = SmartJSValue::new_object(ctx);

    // Check if already exist
    let mut defined_objects = state.defined_objects.borrow_mut();
    if defined_objects.contains_key(type_name) {
        let dobj = defined_objects.get(type_name).unwrap();
        object.set_prop(PXS_PTR_NAME, &mut SmartJSValue::new_i32(ctx, idx));
        // Set proto
        object.set_proto(dobj);
        // DONE
        return object;
    }

    // Create new object
    for object_cbk in source.callbacks.iter() {
        let module_cbk = &object_cbk.cbk;
        let flags = object_cbk.flags;

        let mut func = create_object_callback(ctx, module_cbk.idx, flags);
        // Set
        object.set_prop(&module_cbk.name, &mut func);
    }

    // Define obj
    defined_objects.insert(type_name.clone(), object);

    drop(defined_objects);
    drop(state);

    // Recursion!
    create_object(ctx, idx, source)
}
