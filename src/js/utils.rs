use anyhow::{Result, anyhow};

use crate::{
    borrow_string,
    js::quickjs::{
            self, JS_IsArray, JS_IsError, JS_IsFunction, JS_IsPromise, JS_PromiseResult
        },
    shared::utils::CStringSafe,
};

/// Macro for writing out the JS_Is functions
macro_rules! write_is_func {
    ($($func:ident, $t:ident);*) => {
        $(
            #[allow(unused)]
            #[doc = concat!("Check if Value tag is: ", stringify!($t))]
            pub(super) fn $func(value: &quickjs::JSValue) -> bool {
                (value.tag as i32) == quickjs::$t
            }
        )*
    };
}

/// Macro for writing out the methods for SmartJSValue
macro_rules! write_is_methods {
    ($($func:ident);*) => {
        $(
            #[allow(unused)]
            pub fn $func(&self) -> bool {
                $func(&self.value)
            }
        )*
    };
}

write_is_func! {
    is_undefined, JS_TAG_UNDEFINED;
    is_int, JS_TAG_INT;
    is_string, JS_TAG_STRING;
    is_float, JS_TAG_FLOAT64;
    is_exception, JS_TAG_EXCEPTION;
    is_bool, JS_TAG_BOOL;
    is_object, JS_TAG_OBJECT;
    is_null, JS_TAG_NULL;
    is_bytecode, JS_TAG_FUNCTION_BYTECODE;
    is_module, JS_TAG_MODULE
}

/// Smart JSValue
pub(super) struct SmartJSValue {
    /// The internal value.
    pub value: quickjs::JSValue,
    /// The context that owns the value.
    pub context: *mut quickjs::JSContext,
    /// Own the value (drop it once leave scope)
    pub owned: bool,
}

impl SmartJSValue {
    /// Create a new smart value.
    pub fn new(value: quickjs::JSValue, context: *mut quickjs::JSContext, owned: bool) -> Self {
        Self {
            value,
            context,
            owned,
        }
    }

    /// Create new unonwned value
    pub fn new_borrow(value: quickjs::JSValue, context: *mut quickjs::JSContext) -> Self {
        Self::new(value, context, false)
    }

    /// Create new owned
    pub fn new_owned(value: quickjs::JSValue, context: *mut quickjs::JSContext) -> Self {
        Self::new(value, context, true)
    }

    /// Create new undefined (OWNED)
    pub fn new_undefined(context: *mut quickjs::JSContext) -> Self {
        let v = quickjs::JSValue{
            u: quickjs::JSValueUnion {
                int32: 0
            },
            tag: quickjs::JS_TAG_UNDEFINED as i64,
        };

        Self::new_owned(v, context)
    }

    /// Create a new i32 (OWNED)
    pub fn new_i32(context: *mut quickjs::JSContext, int: i32) -> Self {
        let v = quickjs::JSValue {
            u: quickjs::JSValueUnion {
                int32: int
            },
            tag: quickjs::JS_TAG_INT as i64
        };

        Self::new_owned(v, context)
    }

    /// Create a new f64 (OWNED)
    pub fn new_f64(context: *mut quickjs::JSContext, float: f64) -> Self {
        let v = quickjs::JSValue {
            u: quickjs::JSValueUnion {
                float64: float
            },
            tag: quickjs::JS_TAG_FLOAT64 as i64
        };

        Self::new_owned(v, context)
    }

    /// Create a new boolean (owned)
    pub fn new_bool(context: *mut quickjs::JSContext, val: bool) -> Self {
        let v = quickjs::JSValue {
            u: quickjs::JSValueUnion {
                int32: val as i32
            },
            tag: quickjs::JS_TAG_BOOL as i64
        };

        Self::new_owned(v, context)
    }

    /// Create a new string (owned)
    pub fn new_string(context: *mut quickjs::JSContext, val: String) -> Self {
        let mut cstrgen = CStringSafe::new();
        unsafe {
            let source = cstrgen.new_string(&val);
            let v = quickjs::JS_NewStringLen(context, source, val.len());

            Self::new_owned(v, context)
        }
    }

    /// Create a new Array (owned)
    pub fn new_array(context: *mut quickjs::JSContext) -> Self {
        unsafe {
            let arr = quickjs::JS_NewArray(context);
            Self::new_owned(arr, context)
        }
    }

    /// Create a new null (owned)
    pub fn new_null(context: *mut quickjs::JSContext) -> Self {
        let v = quickjs::JSValue {
            u: quickjs::JSValueUnion {
                int32: 0
            },
            tag: quickjs::JS_TAG_NULL as i64
        };

        Self::new_owned(v, context)
    }

    /// Create a new object (owned)
    pub fn new_object(context: *mut quickjs::JSContext) -> Self {
        unsafe {
            let val = quickjs::JS_NewObject(context);
            Self::new_owned(val, context)
        }
    }

    /// Create a new exception (owned)
    pub fn new_exception(context: *mut quickjs::JSContext, message: String, name: String) -> Self {
        let value = SmartJSValue::new_owned(unsafe {
            quickjs::JS_NewError(context)
        }, context);
        // Dont drop 
        let mut message_val = SmartJSValue::new_string(context, message);
        let mut name_val = SmartJSValue::new_string(context, name);
        value.set_prop("name", &mut name_val);
        value.set_prop("message", &mut message_val);

        return value;
    }

    // /// Create a new object Prototype. Assuming THIS is a prototype. (OWNED)
    // pub fn new_object_proto(&self) -> Self {
    //     SmartJSValue::new_owned(unsafe{quickjs::JS_NewObjectProto(self.context, self.value)}, self.context)
    // }

    #[allow(non_snake_case)]
    /// Get globalThis
    pub fn globalThis(context: *mut quickjs::JSContext) -> Self {
        unsafe {
            let global_this = quickjs::JS_GetGlobalObject(context);
            Self::new_owned(global_this, context)
        }
    }

    #[allow(unused)]
    /// Get current exception
    pub fn current_exception(context: *mut quickjs::JSContext) -> Self {
        unsafe {
            if quickjs::JS_HasException(context) {
                Self::new_owned(quickjs::JS_GetException(context), context)
            } else {
                Self::new_undefined(context)
            }
        }
    }

    #[allow(unused)]
    /// Copy
    pub fn copy(&self) -> Self {
        Self::new_borrow(self.value, self.context)
    }

    /// Get the value duplicated.
    pub fn dupped_value(&self) -> quickjs::JSValue {
        unsafe {
            quickjs::JS_DupValue(self.context, self.value)
        }
    }

    write_is_methods! {
        is_string;
        is_undefined;
        is_float;
        is_int;
        is_exception;
        is_bool;
        is_object;
        is_null;
        is_bytecode;
        is_module
    }

    /// Check is number
    pub fn is_number(&self) -> bool {
        self.is_int() || self.is_float()
    }

    /// Check is error
    pub fn is_error(&self) -> bool {
        unsafe { JS_IsError(self.value) }
    }

    /// Check is function
    pub fn is_function(&self) -> bool {
        unsafe { JS_IsFunction(self.context, self.value) }
    }

    /// Check is array
    pub fn is_array(&self) -> bool {
        unsafe { JS_IsArray(self.value) }
    }

    /// Check if a value is a promise
    pub fn is_promise(&self) -> bool {
        unsafe { JS_IsPromise(self.value) }
    }

    #[allow(unused)]
    /// Get type as string name
    pub fn type_string(&self) -> String {
        match self.value.tag as i32 {
            quickjs::JS_TAG_BIG_INT => "BigInt",
            quickjs::JS_TAG_BOOL => "Bool",
            quickjs::JS_TAG_CATCH_OFFSET => "CatchOffset",
            quickjs::JS_TAG_EXCEPTION => "Exception",
            #[allow(unreachable_patterns)]
            quickjs::JS_TAG_FIRST => "First",
            quickjs::JS_TAG_FLOAT64 => "Float64",
            quickjs::JS_TAG_FUNCTION_BYTECODE => "FunctionBytecode",
            quickjs::JS_TAG_INT => "Int",
            quickjs::JS_TAG_MODULE => "Module",
            quickjs::JS_TAG_NULL => "Null",
            quickjs::JS_TAG_OBJECT => "Object",
            quickjs::JS_TAG_SHORT_BIG_INT => "ShortBigInt",
            quickjs::JS_TAG_STRING => "String",
            quickjs::JS_TAG_STRING_ROPE => "StringRope",
            quickjs::JS_TAG_SYMBOL => "Symbol",
            quickjs::JS_TAG_UNDEFINED => "Undefined",
            quickjs::JS_TAG_UNINITIALIZED => "Unitialized",
            _ => "Unkown"
        }.to_string()
    }

    /// Await the promise
    /// 
    /// Returns OWNED
    pub fn await_value(&self) -> Self {
        if !self.is_promise() {
            Self::new_undefined(self.context)
        } else {
            unsafe {
                Self::new_owned(JS_PromiseResult(self.context, self.value), self.context)
            }
        }
    }

    /// Get As String (only works on strings)
    pub fn as_string(&self) -> Result<String> {
        if !self.is_string() {
            return Err(anyhow!("JSValue is not a string"));
        }

        unsafe {
            let cstring =
                quickjs::JS_ToCStringLen2(self.context, std::ptr::null_mut(), self.value, false);
            if cstring.is_null() {
                Err(anyhow!("String result is NULL"))
            } else {
                let val = borrow_string!(cstring).to_string();
                quickjs::JS_FreeCString(self.context, cstring);
                Ok(val)
            }
        }
    }

    /// Get as i32 (only works on numbers)
    pub fn as_i32(&self) -> Result<i32> {
        if !self.is_number() {
            return Err(anyhow!("JSValue is not a i32"));
        }

        unsafe {
            let mut int32 = -1;
            quickjs::JS_ToInt32(self.context, &mut int32, self.value);
            Ok(int32)
        }
    }

    /// Get as f64 (only works on numbers)
    pub fn as_f64(&self) -> Result<f64> {
        if !self.is_number() {
            return Err(anyhow!("JSValue is not a f64"));
        }
        
        unsafe {
            let mut float = -1.0f64;
            quickjs::JS_ToFloat64(self.context, &mut float, self.value);
            Ok(float)
        }
    }

    /// Get as bool
    pub fn as_bool(&self) -> bool {
        unsafe {
            let i = quickjs::JS_ToBool(self.context, self.value);
            i == 1
        }
    }

    /// Get a property off a Value.
    pub fn get_prop<T: ToString>(&self, key: T) -> Self {
        unsafe {
            let mut cstrgen = CStringSafe::new();
            let prop = quickjs::JS_GetPropertyStr(
                self.context,
                self.value,
                cstrgen.new_string(&key.to_string()),
            );
            Self::new_owned(prop, self.context)
        }
    }

    /// Get a property off a Value.
    pub fn get_prop_pos(&self, key: u32) -> Self {
        unsafe {
            let prop = quickjs::JS_GetPropertyUint32(self.context, self.value, key);
            Self::new_owned(prop, self.context)
        }
    }

    /// Get module pointer.
    /// 
    /// BORROW
    pub fn get_module_ptr(&self) -> *mut quickjs::JSModuleDef {
        if !self.is_module() {
            std::ptr::null_mut()
        } else {
            let val_int = unsafe { self.value.u.ptr } as isize;
            ((val_int & !15) as *mut std::ffi::c_void).cast::<quickjs::JSModuleDef>()
        }
    }

    #[allow(unused)]
    /// Get module namespace
    /// 
    /// OWNED
    pub fn get_module_namespace(&self) -> Self {
        if !self.is_module() {
            Self::new_undefined(self.context)
        } else {
            let m = self.get_module_ptr();
            if m == std::ptr::null_mut() {
                Self::new_undefined(self.context)
            } else {
                Self::new_owned(unsafe{quickjs::JS_GetModuleNamespace(self.context, m)}, self.context)
            }
        }
    }

    /// Set a property.
    /// 
    /// Un owns value
    pub fn set_prop<T: ToString>(&self, key: T, value: &mut SmartJSValue) {
        value.owned = false;
        unsafe {
            let mut cstrgen = CStringSafe::new();
            quickjs::JS_SetPropertyStr(
                self.context,
                self.value,
                cstrgen.new_string(&key.to_string()),
                value.value,
            );
        }
    }

    /// Set a property
    /// 
    /// Un owns property
    pub fn set_prop_pos(&self, key: u32, value: &mut SmartJSValue) {
        value.owned = false;
        unsafe {
            quickjs::JS_SetPropertyUint32(self.context, self.value, key, value.value);
        }
    }

    /// Set a property
    /// 
    /// Un own property
    pub fn set_prop_value(&self, key: &SmartJSValue, value: &mut SmartJSValue) {
        value.owned = false;
        unsafe {
            let atom = quickjs::JS_ValueToAtom(self.context, key.value);
            quickjs::JS_SetProperty(self.context, self.value, atom, value.value);
            quickjs::JS_FreeAtom(self.context, atom);
        }
    }

    /// Remove a property
    pub fn del_prop(&self, prop: &SmartJSValue) {
        unsafe {
            let atom = quickjs::JS_ValueToAtom(self.context, prop.value);
            quickjs::JS_DeleteProperty(self.context, self.value, atom, 0);
            quickjs::JS_FreeAtom(self.context, atom);
        }
    }

    /// Set Prototype
    pub fn set_proto(&self, proto: &SmartJSValue) {
        unsafe {
            quickjs::JS_SetPrototype(self.context, self.value, proto.value);
        }
    }

    /// Add a Getter & Setter
    pub fn add_getter_setter(&self, property: &str, cbk: &SmartJSValue) {
        let mut cstrsafe = CStringSafe::new();
        unsafe {
            // Atomize
            let atom = quickjs::JS_NewAtom(self.context, cstrsafe.new_string(property));

            let getter = cbk.dupped_value();
            let setter = cbk.dupped_value();
            quickjs::JS_DefinePropertyGetSet(self.context, self.value, atom, getter, setter, (quickjs::JS_PROP_CONFIGURABLE | quickjs::JS_PROP_ENUMERABLE) as i32);

            quickjs::JS_FreeAtom(self.context, atom);
        }
    }

    /// Get a Error/Exception
    pub fn get_error_exception(&self) -> Option<String> {
        if !self.is_error() && !self.is_exception() {
            return None;
        }

        if self.is_exception() {
            let message = Self::current_exception(self.context).to_string_direct();
            let stack = self.get_prop("stack").to_string_direct();
            return Some(format!("Exception: {message}, stack: {stack}"));
        }

        let message = self.get_prop("message");
        let name = self.get_prop("name");
        let stack = self.get_prop("stack");

        // Result
        let res = format!(
            "{}, {}, {}",
            name.as_string().unwrap_or("Error".to_string()),
            message.as_string().unwrap_or("Unkown Error Message".to_string()),
            stack.as_string().unwrap_or("Uknown stack".to_string())
        );
        Some(res)
    }

    #[allow(unused)]
    /// ToStringDirect (no checks)
    pub fn to_string_direct(&self) -> String {
        unsafe {
            let val = SmartJSValue::new_owned(quickjs::JS_ToString(self.context, self.value), self.context);
            val.as_string().unwrap_or("ERROR".to_string())
        }
    }

    #[allow(unused)]
    /// ToString
    pub fn to_string(&self) -> String {
        // already string
        if self.is_string() {
            return self.as_string().unwrap();
        }

        unsafe {
            // Convert
            let val = SmartJSValue::new_owned(
                quickjs::JS_ToString(self.context, self.value),
                self.context,
            );

            if val.is_error() || val.is_exception() {
                return self.get_error_exception().unwrap();
            }

            val.as_string().unwrap_or("[Invalid String]".to_string())
        }
    }

    /// Call a function on this value.
    ///
    /// Returns OWNED value.
    pub fn call<T: ToString>(&self, name: T, args: &Vec<SmartJSValue>) -> SmartJSValue {
        let mut js_args = vec![];
        for i in args.iter() {
            js_args.push(i.value);
        }
        let argv = js_args.as_mut_ptr();
        unsafe {
            let function = self.get_prop(name);
            if !function.is_function() {
                return SmartJSValue::new_exception(self.context, "Not a function".to_string(), "CallFunctionException".to_string());
            }
            let result = SmartJSValue::new_owned(
                quickjs::JS_Call(self.context, function.value, self.value, args.len().try_into().unwrap(), argv),
                self.context,
            );
            result
        }
    }

    /// Call this value as a function.
    /// 
    /// Returns OWNED value.
    pub fn call_as_source(&self, args: &Vec<SmartJSValue>) -> SmartJSValue {
        if !self.is_function() {
            return SmartJSValue::new_exception(self.context, "Not a function".to_string(), "CallFunctionException".to_string());
        }
        
        let mut js_args = vec![];
        for i in args.iter() {
            js_args.push(i.value);
        }
        let argv = js_args.as_mut_ptr();
        unsafe {
            let undefined = SmartJSValue::new_undefined(self.context);
            let result = SmartJSValue::new_owned(
                quickjs::JS_Call(self.context, self.value, undefined.value, args.len().try_into().unwrap(), argv),
                self.context,
            );
            result
        }
    }
}

impl Clone for SmartJSValue {
    fn clone(&self) -> Self {
        // Duplicate the value
        Self::new_owned(self.dupped_value(), self.context)
    }
}

impl Drop for SmartJSValue {
    fn drop(&mut self) {
        // Dont free when owned.
        if !self.owned || self.context.is_null() {
            return;
        }
        // Free
        unsafe {
            quickjs::JS_FreeValue(self.context, self.value);
            self.context = std::ptr::null_mut();
        }
    }
}