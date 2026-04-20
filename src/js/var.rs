// Convert PXS vars to JS vars.
// Convert JS vars to PXS vars.

use std::{ffi::c_void, ptr::NonNull};

use rquickjs::{Array, Ctx, Error, IntoJs, Object, Result, Value, qjs};

use crate::{pxs_debug, shared::{
    PtrMagic, pxs_Runtime, var::{pxs_Var, pxs_VarObject}
}};

/// JS PXS Container.
/// Holds the context that the Value is made from.
struct JSPXSContaner {
    context: *mut qjs::JSContext,
    value: *mut c_void,
    /// Either true for Object, or false for function
    is_object: bool,
}

impl JSPXSContaner {
    /// Create a new JSPXSContainer from a Value.
    pub unsafe fn from_value(value: Value, is_object: bool) -> Self {
        let raw_ptr: qjs::JSValue = value.as_raw();

        unsafe {
            let context = value.ctx().as_raw().as_ptr();
            // Dup it
            qjs::JS_DupValue(context, raw_ptr);

            let ptr = raw_ptr.u.ptr;
            JSPXSContaner {
                context: context,
                value: ptr,
                is_object: is_object,
            }
        }
    }

    /// Recreate the JSValue (qjs)
    pub fn recreate(self: &Self) -> qjs::JSValue {
        qjs::JSValue {
            u: qjs::JSValueUnion {
                ptr: self.value,
            },
            tag: qjs::JS_TAG_OBJECT as i64,
        }
    }

    /// Send into a `rquickjs::Value`
    pub fn into_value<'js>(self: &Self) -> Result<Value<'js>> {
        unsafe {
            let val = self.recreate();
            let non_null_context = NonNull::new(self.context).unwrap();
            let ctx = Ctx::from_raw(non_null_context);
            Ok(Value::from_raw(ctx, val))
        }
    }
}

impl PtrMagic for JSPXSContaner {}

/// JS Object deleters
unsafe extern "C" fn js_deleter(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }

    // We will be dropping this dude.
    let container = JSPXSContaner::from_raw(ptr as *mut JSPXSContaner);

    // construct the value to drop it
    unsafe {
        let val = container.recreate();
        // Free the value
        qjs::JS_FreeValue(container.context, val);
    }
}

/// Convert a JS Value into a pxs_Var
pub(super) fn js_into_pxs(value: Value) -> rquickjs::Result<pxs_Var> {
    if value.is_int() {
        Ok(pxs_Var::new_i64(value.as_int().unwrap_or_default().into()))
    } else if value.is_float() {
        Ok(pxs_Var::new_f64(
            value.as_float().unwrap_or_default().into(),
        ))
    } else if value.is_bool() {
        Ok(pxs_Var::new_bool(value.as_bool().unwrap_or_default()))
    } else if value.is_string() {
        Ok(pxs_Var::new_string(value.as_string().unwrap().to_string()?))
    } else if value.is_array() {
        let mut values = vec![];
        let arr = value.as_array().unwrap();

        for i in 0..arr.len() {
            let val: Value = arr.get(i)?;
            values.push(js_into_pxs(val)?);
        }

        Ok(pxs_Var::new_list_with(values))
    } else if value.is_function() {
        unsafe {
            Ok(pxs_Var::new_object(
                pxs_VarObject::new_lang_only(
                    JSPXSContaner::from_value(value, false).into_raw() as *mut c_void
                ),
                Some(js_deleter),
            ))
        }
    } else if value.is_object() {
        unsafe {
            Ok(pxs_Var::new_object(
                pxs_VarObject::new_lang_only(
                    JSPXSContaner::from_value(value, true).into_raw() as *mut c_void
                ),
                Some(js_deleter),
            ))
        }
    } 
    else if value.is_exception() {
        let exce = value.as_exception().unwrap();
        Ok(pxs_Var::new_exception(exce.message().unwrap()))
    } else if value.is_error() {
        let error = value.as_object().unwrap();
        let message: rquickjs::String = error.get("message")?;
        let name: rquickjs::String = error.get("name")?;
        
        Ok(pxs_Var::new_exception(format!("{:#?}:{:#?}", name.to_string()?, message.to_string()?)))
    } else {
        // null, undefined, etc.
        Ok(pxs_Var::new_null())
    }
}

/// Convert a `pxs_Var` into a JS Value.
pub(super) fn pxs_into_js<'js>(ctx: &Ctx<'js>, var: &pxs_Var) -> Result<Value<'js>> {
    match var.tag {
        crate::shared::var::pxs_VarType::pxs_Int64 => var.get_i64().unwrap().into_js(ctx),
        crate::shared::var::pxs_VarType::pxs_UInt64 => var.get_u64().unwrap().into_js(ctx),
        crate::shared::var::pxs_VarType::pxs_String => var.get_string().unwrap().into_js(ctx),
        crate::shared::var::pxs_VarType::pxs_Bool => var.get_bool().unwrap().into_js(ctx),
        crate::shared::var::pxs_VarType::pxs_Float64 => var.get_f64().unwrap().into_js(ctx),
        crate::shared::var::pxs_VarType::pxs_Null => Ok(Value::new_null(ctx.clone())),
        crate::shared::var::pxs_VarType::pxs_Object => {
            // Pass pointer back
            let container_ptr = var.get_object_ptr();
            if container_ptr.is_null() {
                // I want to return Exception("Object pointer not found.")
                return Err(ctx.throw("Object pointer not found".into_js(ctx)?));
            }
            let container = unsafe{JSPXSContaner::from_borrow_void(container_ptr)};

            // Return a Value. (Do not perform duplication. Duplication is only performed from quick -> pxs not the other way around.)
            container.into_value()
        },
        crate::shared::var::pxs_VarType::pxs_HostObject => todo!(),
        crate::shared::var::pxs_VarType::pxs_List => {
            let arr = Array::new(ctx.clone())?;

            let vars = &var.get_list().unwrap().vars;
            for i in 0..vars.len() {
                // convert to JS value
                let value = pxs_into_js(ctx, &vars[i])?;
                arr.set(i, value);
            }

            Ok(arr.into_value())
        },
        crate::shared::var::pxs_VarType::pxs_Function => {
            // What?
            let container_ptr = var.get_function().unwrap();
            if container_ptr.is_null() {
                return Err(ctx.throw("Function pointer not found".into_js(ctx)?));
            }
            let container = unsafe{JSPXSContaner::from_borrow_void(container_ptr)};
            container.into_value()
        },
        crate::shared::var::pxs_VarType::pxs_Factory => {
            // Call and return
            let factory = var.get_factory().unwrap();
            let res = factory.call(pxs_Runtime::pxs_JavaScript);
            // convert into js
            pxs_into_js(ctx, &res)
        },
        crate::shared::var::pxs_VarType::pxs_Exception => {
            Err(ctx.throw(var.get_string().unwrap().into_js(ctx)?))
        },
        crate::shared::var::pxs_VarType::pxs_Map => {
            let object = Object::new(ctx.clone())?;
            
            let map = var.get_map().unwrap();
            let keys = map.keys();

            for k in keys {
                let item = map.get_item(k);
                if let Some(item) = item {
                    let js_key = pxs_into_js(ctx, k)?;
                    let js_val = pxs_into_js(ctx, item)?;

                    object.set(js_key, js_val)?;
                }
            }
            
            Ok(object.into_value())
        },
    }
}
