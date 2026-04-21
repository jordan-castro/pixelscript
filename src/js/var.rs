// Convert PXS vars to JS vars.
// Convert JS vars to PXS vars.

use std::ffi::c_void;
use anyhow::{Result, anyhow};

use crate::{js::{SmartJSValue, quickjs, register_add_object, register_del_object, register_get_object}, pxs_debug, shared::{
    PtrMagic, pxs_Runtime, var::{pxs_Var, pxs_VarObject}
}};

/// JS PXS Container.
/// Holds the context that the Value is made from.
struct JSPXSContainer {
    /// The value (idx)
    ptr: i32,
    context: *mut quickjs::JSContext
}

impl JSPXSContainer {
    /// Create a new JSPXSContainer from a Value.
    pub fn from_value(value: SmartJSValue) -> Self {
        let ptr = register_add_object(value.clone());
        if ptr.is_err() {
            JSPXSContainer { ptr: -1, context: value.context }
        } else {
            JSPXSContainer { ptr: ptr.unwrap(), context: value.context }
        }
    }

    /// Get the SmartJSValue from the registry
    pub fn get_value(&self) -> Result<SmartJSValue> {
        if self.ptr < 0 {
            pxs_debug!("JSPXSContainer is empty.");
            Err(anyhow!("JSPXSContainer is empty."))
            // SmartJSValue::new_undefined(self.context)
        } else {
            Ok(register_get_object(self.ptr))
        }
    } 
}

impl PtrMagic for JSPXSContainer {}

/// JS Object deleters
unsafe extern "C" fn js_deleter(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }

    // We will be dropping this dude.
    let container = JSPXSContainer::from_raw(ptr as *mut JSPXSContainer);

    // construct the value to drop it
    register_del_object(container.ptr);
}

/// Convert a JS Value into a pxs_Var
pub(super) fn js_into_pxs(value: &SmartJSValue) -> Result<pxs_Var> {
    if value.is_int() {
        Ok(pxs_Var::new_i64(value.as_i32()? as i64))
    } else if value.is_float() {
        Ok(pxs_Var::new_f64(
            value.as_f64()?,
        ))
    } else if value.is_bool() {
        Ok(pxs_Var::new_bool(value.as_bool()))
    } else if value.is_string() {
        Ok(pxs_Var::new_string(value.as_string()?))
    } else if value.is_array() {
        let mut values = vec![];
        let length = value.get_prop("length").as_i32()?;

        for i in 0..length {
            let val = value.get_prop_pos(i as u32);
            values.push(js_into_pxs(&val)?);
        }

        Ok(pxs_Var::new_list_with(values))
    } else if value.is_function() {
        Ok(pxs_Var::new_object(
            pxs_VarObject::new_lang_only(
                JSPXSContainer::from_value(value.copy()).into_raw() as *mut c_void
            ),
            Some(js_deleter),
        ))
    } 
    else if value.is_exception() {
        // let exce = value.as_exception().unwrap();
        Ok(pxs_Var::new_exception(value.get_error_exception().unwrap()))
    } else if value.is_error() {
        Ok(pxs_Var::new_exception(value.get_error_exception().unwrap()))
    } else if value.is_object() {
        Ok(pxs_Var::new_object(
            pxs_VarObject::new_lang_only(
                JSPXSContainer::from_value(value.copy()).into_raw() as *mut c_void
            ),
            Some(js_deleter),
        ))
    } else {
        // null, undefined, etc.
        Ok(pxs_Var::new_null())
    }
}

/// Convert a `pxs_Var` into a JS Value.
pub(super) fn pxs_into_js(context: *mut quickjs::JSContext, var: &pxs_Var) -> Result<SmartJSValue> {
    match var.tag {
        crate::shared::var::pxs_VarType::pxs_Int64 => Ok(SmartJSValue::new_i32(context, var.get_i64().unwrap() as i32)),
        // TODO: support UInt
        crate::shared::var::pxs_VarType::pxs_UInt64 => Ok(SmartJSValue::new_i32(context, var.get_u64().unwrap() as i32)),
        crate::shared::var::pxs_VarType::pxs_String => Ok(SmartJSValue::new_string(context, var.get_string().unwrap())),
        crate::shared::var::pxs_VarType::pxs_Bool => Ok(SmartJSValue::new_bool(context, var.get_bool().unwrap())),
        crate::shared::var::pxs_VarType::pxs_Float64 => Ok(SmartJSValue::new_f64(context, var.get_f64().unwrap())),
        crate::shared::var::pxs_VarType::pxs_Null => Ok(SmartJSValue::new_null(context)),
        crate::shared::var::pxs_VarType::pxs_Object => {
            // Pass pointer back
            let container_ptr = var.get_object_ptr();
            if container_ptr.is_null() {
                // I want to return Exception("Object pointer not found.")
                return Err(anyhow!("Object pointer not found"));
            }
            let container = unsafe{JSPXSContainer::from_borrow_void(container_ptr)};

            // Return a Value. (Do not perform duplication. Duplication is only performed from quick -> pxs not the other way around.)
            container.get_value()
        },
        crate::shared::var::pxs_VarType::pxs_HostObject => todo!(),
        crate::shared::var::pxs_VarType::pxs_List => {
            let arr = SmartJSValue::new_array(context);
            let vars = &var.get_list().unwrap().vars;
            for i in 0..vars.len() {
                // convert to JS value
                let value = pxs_into_js(context, &vars[i])?;
                arr.set_prop_pos(i as u32, &value);
            }
            
            Ok(arr)
        },
        crate::shared::var::pxs_VarType::pxs_Function => {
            // What?
            let container_ptr = var.get_function().unwrap();
            if container_ptr.is_null() {
                return Err(anyhow!("Function pointer not found"));
            }
            let container = unsafe{JSPXSContainer::from_borrow_void(container_ptr)};
            container.get_value()
        },
        crate::shared::var::pxs_VarType::pxs_Factory => {
            // Call and return
            let factory = var.get_factory().unwrap();
            let res = factory.call(pxs_Runtime::pxs_JavaScript);
            // convert into js
            pxs_into_js(context, &res)
        },
        crate::shared::var::pxs_VarType::pxs_Exception => {
            Err(anyhow!("{}", var.get_string().unwrap()))
        },
        crate::shared::var::pxs_VarType::pxs_Map => {
            let object = SmartJSValue::new_object(context);
            
            let map = var.get_map().unwrap();
            let keys = map.keys();

            for k in keys {
                let item = map.get_item(k);
                if let Some(item) = item {
                    let js_key = pxs_into_js(context, k)?;
                    let js_val = pxs_into_js(context, item)?;

                    object.set_prop_value(&js_key, &js_val);
                }
            }
            
            Ok(object)
        },
    }
}
