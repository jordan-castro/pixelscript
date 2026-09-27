//! QuickjsNG bindings. Has support for emscripten!

// == GLOBALS ==

pub const JS_EVAL_TYPE_GLOBAL: core::ffi::c_int = 0 << 0; /* global code (default) */
pub const JS_EVAL_TYPE_MODULE: core::ffi::c_int = 1 << 0; /* module code */
pub const JS_EVAL_FLAG_COMPILE_ONLY: core::ffi::c_int = 1 << 5;
/* all tags with a reference count are negative */
pub const JS_TAG_FIRST: core::ffi::c_int = -9; /* first negative tag */
pub const JS_TAG_BIG_INT: core::ffi::c_int = -9;
pub const JS_TAG_SYMBOL: core::ffi::c_int = -8;
pub const JS_TAG_STRING: core::ffi::c_int = -7;
pub const JS_TAG_STRING_ROPE: core::ffi::c_int = -6;
pub const JS_TAG_MODULE: core::ffi::c_int = -3; /* used internally */
pub const JS_TAG_FUNCTION_BYTECODE: core::ffi::c_int = -2; /* used internally */
pub const JS_TAG_OBJECT: core::ffi::c_int = -1;
pub const JS_TAG_INT: core::ffi::c_int = 0;
pub const JS_TAG_BOOL: core::ffi::c_int = 1;
pub const JS_TAG_NULL: core::ffi::c_int = 2;
pub const JS_TAG_UNDEFINED: core::ffi::c_int = 3;
pub const JS_TAG_UNINITIALIZED: core::ffi::c_int = 4;
pub const JS_TAG_CATCH_OFFSET: core::ffi::c_int = 5;
pub const JS_TAG_EXCEPTION: core::ffi::c_int = 6;
pub const JS_TAG_SHORT_BIG_INT: core::ffi::c_int = 7;
pub const JS_TAG_FLOAT64: core::ffi::c_int = 8;

pub const JS_FLOAT64_TAG_ADDEND: core::ffi::c_int = 0x7ff80000 - JS_TAG_FIRST + 1;
pub const JS_NAN: u64 = 0x7ff8000000000000_u64.wrapping_sub((JS_FLOAT64_TAG_ADDEND as u64) << 32);

pub const JS_PROP_CONFIGURABLE: core::ffi::c_int = 1 << 0;
pub const JS_PROP_ENUMERABLE: core::ffi::c_int = 1 << 2;

// == GLOBALS END ==

// == TYPES ==

/// The JSRuntime.
#[repr(C)]
pub struct JSRuntime {
    _unused: [u8; 0],
}

/// The JSContext.
#[repr(C)]
pub struct JSContext {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct JSModuleDef {
    _unused: [u8; 0],
}

#[cfg(target_os = "emscripten")]
pub type JSValue = u64;

#[cfg(not(target_os = "emscripten"))]
#[repr(C)]
#[derive(Clone, Copy)]
pub union JSValueUnion {
    pub int32: i32,
    pub float64: core::ffi::c_double,
    pub ptr: *mut core::ffi::c_void,
    pub short_big_int: i32,
}

#[cfg(not(target_os = "emscripten"))]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JSValue {
    pub u: JSValueUnion,
    pub tag: i64,
}

pub type JSValueConst = JSValue;

pub type JSAtom = u32;

pub type JSModuleNormalizeFunc = unsafe extern "C" fn(
    ctx: *mut JSContext,
    mbn: *const core::ffi::c_char,
    mn: *const core::ffi::c_char,
    opaque: *mut core::ffi::c_void,
) -> *mut core::ffi::c_char;

pub type JSModuleLoaderFunc = unsafe extern "C" fn(
    ctx: *mut JSContext,
    module_name: *const core::ffi::c_char,
    opaque: *mut core::ffi::c_void,
) -> *mut JSModuleDef;

pub type JSCFunction = unsafe extern "C" fn(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    argc: core::ffi::c_int,
    argv: *mut JSValueConst,
) -> JSValue;

#[cfg(target_os = "emscripten")]
pub union JSValueFloat64Union {
    pub d: core::ffi::c_double,
    pub un: u64,
}

#[repr(i32)]
pub enum JSCFunctionEnum {
    /* XXX: should rename for namespace isolation */
    JS_CFUNC_generic = 0,
    JS_CFUNC_generic_magic,
    JS_CFUNC_constructor,
    JS_CFUNC_constructor_magic,
    JS_CFUNC_constructor_or_func,
    JS_CFUNC_constructor_or_func_magic,
    JS_CFUNC_f_f,
    JS_CFUNC_f_f_f,
    JS_CFUNC_getter,
    JS_CFUNC_setter,
    JS_CFUNC_getter_magic,
    JS_CFUNC_setter_magic,
    JS_CFUNC_iterator_next,
}

pub type JSCFunctionData = unsafe extern "C" fn(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    argc: core::ffi::c_int,
    argv: *mut JSValueConst,
    magic: core::ffi::c_int,
    func_data: *mut JSValueConst,
) -> JSValue;

pub type JSModuleInitFunc =
    unsafe extern "C" fn(ctx: *mut JSContext, m: *mut JSModuleDef) -> core::ffi::c_int;

// == TYPES END ==

// == FUNCTIONS ==

unsafe extern "C" {
    pub fn JS_NewRuntime() -> *mut JSRuntime;
    pub fn JS_NewContext(rt: *mut JSRuntime) -> *mut JSContext;
    pub fn JS_SetModuleLoaderFunc(
        rt: *mut JSRuntime,
        module_normalize: Option<JSModuleNormalizeFunc>,
        module_loader: Option<JSModuleLoaderFunc>,
        opaque: *mut core::ffi::c_void,
    );
    pub fn JS_NewCFunction2(
        ctx: *mut JSContext,
        func: Option<JSCFunction>,
        name: *const core::ffi::c_char,
        length: core::ffi::c_int,
        cproto: JSCFunctionEnum,
        magic: core::ffi::c_int,
    ) -> JSValue;
    pub fn JS_FreeContext(s: *mut JSContext);
    pub fn JS_FreeRuntime(rt: *mut JSRuntime);
    pub fn JS_Eval(
        ctx: *mut JSContext,
        input: *const core::ffi::c_char,
        input_len: usize,
        filename: *const core::ffi::c_char,
        eval_flags: core::ffi::c_int,
    ) -> JSValue;
    pub fn JS_ThrowInternalError(
        ctx: *mut JSContext,
        fmt: *const core::ffi::c_char,
        ...
    ) -> JSValue;
    pub fn JS_GetModuleNamespace(ctx: *mut JSContext, m: *mut JSModuleDef) -> JSValue;
    pub fn JS_EvalFunction(ctx: *mut JSContext, fun_obj: JSValue) -> JSValue;
    pub fn JS_RunGC(rt: *mut JSRuntime);
    pub fn JS_Throw(ctx: *mut JSContext, obj: JSValue) -> JSValue;
    pub fn JS_NewCFunctionData(
        ctx: *mut JSContext,
        func: Option<JSCFunctionData>,
        length: core::ffi::c_int,
        magic: core::ffi::c_int,
        data_len: core::ffi::c_int,
        data: *mut JSValueConst,
    ) -> JSValue;
    pub fn JS_IsArray(val: JSValueConst) -> bool;
    pub fn JS_IsError(val: JSValueConst) -> bool;
    pub fn JS_IsFunction(ctx: *mut JSContext, val: JSValueConst) -> bool;
    pub fn JS_IsPromise(val: JSValueConst) -> bool;
    pub fn JS_PromiseResult(ctx: *mut JSContext, promise: JSValueConst) -> JSValue;
    pub fn JS_ToFloat64(
        ctx: *mut JSContext,
        pres: *mut core::ffi::c_double,
        val: JSValueConst,
    ) -> core::ffi::c_int;
    pub fn JS_NewStringLen(
        ctx: *mut JSContext,
        str1: *const core::ffi::c_char,
        len1: usize,
    ) -> JSValue;
    pub fn JS_NewArray(ctx: *mut JSContext) -> JSValue;
    pub fn JS_NewObject(ctx: *mut JSContext) -> JSValue;
    pub fn JS_NewError(ctx: *mut JSContext) -> JSValue;
    pub fn JS_GetGlobalObject(ctx: *mut JSContext) -> JSValue;
    pub fn JS_HasException(ctx: *mut JSContext) -> bool;
    pub fn JS_GetException(ctx: *mut JSContext) -> JSValue;
    pub fn JS_DupValue(ctx: *mut JSContext, v: JSValueConst) -> JSValue;
    pub fn JS_FreeValue(ctx: *mut JSContext, v: JSValue);
    pub fn JS_ToCStringLen2(
        ctx: *mut JSContext,
        plen: *mut usize,
        val1: JSValueConst,
        cesu8: bool,
    ) -> *const core::ffi::c_char;
    pub fn JS_FreeCString(ctx: *mut JSContext, ptr: *const core::ffi::c_char);
    pub fn JS_ToInt32(ctx: *mut JSContext, pres: *mut i32, val: JSValueConst) -> core::ffi::c_int;
    pub fn JS_ToBool(ctx: *mut JSContext, val: JSValueConst) -> core::ffi::c_int;
    pub fn JS_GetPropertyStr(
        ctx: *mut JSContext,
        this_obj: JSValueConst,
        prop: *const core::ffi::c_char,
    ) -> JSValue;
    pub fn JS_GetPropertyUint32(ctx: *mut JSContext, this_obj: JSValueConst, idx: u32) -> JSValue;
    pub fn JS_SetPropertyStr(
        ctx: *mut JSContext,
        this_obj: JSValueConst,
        prop: *const core::ffi::c_char,
        val: JSValue,
    ) -> core::ffi::c_int;
    pub fn JS_SetPropertyUint32(
        ctx: *mut JSContext,
        this_obj: JSValueConst,
        idx: u32,
        val: JSValue,
    ) -> core::ffi::c_int;
    pub fn JS_ValueToAtom(ctx: *mut JSContext, val: JSValueConst) -> JSAtom;

    pub fn JS_SetProperty(
        ctx: *mut JSContext,
        this_obj: JSValueConst,
        prop: JSAtom,
        val: JSValue,
    ) -> core::ffi::c_int;
    pub fn JS_FreeAtom(ctx: *mut JSContext, v: JSAtom);
    pub fn JS_DeleteProperty(
        ctx: *mut JSContext,
        obj: JSValueConst,
        prop: JSAtom,
        flags: core::ffi::c_int,
    ) -> core::ffi::c_int;
    pub fn JS_SetPrototype(
        ctx: *mut JSContext,
        obj: JSValueConst,
        proto_val: JSValueConst,
    ) -> core::ffi::c_int;
    pub fn JS_NewAtom(ctx: *mut JSContext, str: *const core::ffi::c_char) -> JSAtom;
    pub fn JS_DefinePropertyGetSet(
        ctx: *mut JSContext,
        this_obj: JSValueConst,
        prop: JSAtom,
        getter: JSValue,
        setter: JSValue,
        flags: core::ffi::c_int,
    ) -> core::ffi::c_int;
    pub fn JS_ToString(ctx: *mut JSContext, val: JSValueConst) -> JSValue;
    pub fn JS_Call(
        ctx: *mut JSContext,
        func_obj: JSValueConst,
        this_obj: JSValueConst,
        argc: core::ffi::c_int,
        argv: *mut JSValueConst,
    ) -> JSValue;
    pub fn JS_GetModuleName(ctx: *mut JSContext, m: *mut JSModuleDef) -> JSAtom;
    pub fn JS_AtomToCStringLen(
        ctx: *mut JSContext,
        plen: *mut usize,
        atom: JSAtom,
    ) -> *const core::ffi::c_char;
    pub fn JS_NewCModule(
        ctx: *mut JSContext,
        name_str: *const core::ffi::c_char,
        func: Option<JSModuleInitFunc>,
    ) -> *mut JSModuleDef;
    /* can only be called before the module is instantiated */
    pub fn JS_AddModuleExport(
        ctx: *mut JSContext,
        m: *mut JSModuleDef,
        name_str: *const core::ffi::c_char,
    ) -> core::ffi::c_int;
    pub fn JS_SetModuleExport(
        ctx: *mut JSContext,
        m: *mut JSModuleDef,
        export_name: *const core::ffi::c_char,
        val: JSValue,
    ) -> core::ffi::c_int;
    pub fn JS_ThrowReferenceError(ctx: *mut JSContext, fmt: *const core::ffi::c_char, ...) -> JSValue;
}

// == FUNCTIONS END ==
