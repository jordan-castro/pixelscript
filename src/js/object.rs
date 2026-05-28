use std::sync::Arc;

use crate::{js::{SmartJSValue, func::create_object_callback, get_js_state, quickjs}, shared::{PXS_PTR_NAME, object::{ObjectFlags, pxs_PixelObject}}};

pub(super) fn create_object(ctx: *mut quickjs::JSContext, idx: i32, source: Arc<pxs_PixelObject>) -> SmartJSValue {
    let state = get_js_state();
    let type_name = &source.type_name;

    let object = SmartJSValue::new_object(ctx);

    // Check if already exist
    // let mut defined_objects = state.defined_objects.borrow_mut();
    unsafe { 
        if let Some(dobj) = (*state).defined_objects.get(type_name) {
            object.set_prop(PXS_PTR_NAME, &mut SmartJSValue::new_i32(ctx, idx));
            // Set proto
            object.set_proto(dobj);
            return object;
        }
    }

    // Create new object
    for object_cbk in source.callbacks.iter() {
        let module_cbk = &object_cbk.cbk;
        let flags = object_cbk.flags;

        let mut func = create_object_callback(ctx, module_cbk.idx, flags);
        if flags & (ObjectFlags::IsProp as u8) != 0 {
            object.add_getter_setter(&module_cbk.name, &func);
        } else {
            // Set
            object.set_prop(&module_cbk.name, &mut func);
        }
    }

    // Define obj
    unsafe {
        (*state).defined_objects.insert(type_name.clone(), object);
    }

    // Recursion!
    create_object(ctx, idx, source)
}
