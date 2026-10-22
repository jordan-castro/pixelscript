//!
//! from pxs.cross import rjs
//!
//! # from js
//! eval_res = rjs._eval("1 + 1")
//! # run js code
//! rjs._exec("""
//! import {print} from 'pxs';
//! function hello(name) {
//!     print(name);
//! }
//!
//! const Person = {
//!     name: "Jordan",
//!     foo: () => {
//!         print('Foo')
//!     }
//! };
//! """)
//!
//! # Grab hello func
//! hello_func = rjs._get('hello')
//! # Call it
//! hello_func._call(["Jordan"])
//!
//! # Call directly without grabbing
//! rjs._call("hello", ["Jordan"])
//!
//! # Grab object
//! person_obj = rjs._get('Person')
//!
//! # get name
//! print(person_obj._get('name'))
//!
//! # call foo
//! person_obj._call('foo')
//!
//! # Passing objects back and forth is UB.
//! # if you must pass objects, then expect them like so
//! rjs._exec("""
//! const expects_object = (obj) => {
//!     return obj._get('name')
//! }
//! """)
//!
//! class Person:
//!     def __init__(self, name):
//!         self.name = name
//!
//! print(rjs._call('expects_object', [Person('Jordan')]))

use etffi::{cstring::CStringSafe, own_string, ptr_magic::PtrMagic};

use crate::{
    get_self, pxs_add_submod, pxs_addfunc, pxs_arena_putstr, pxs_arenaput, pxs_arg, pxs_argc, pxs_call, pxs_core::PxsCoreType, pxs_evalnamed, pxs_exec, pxs_freearena, pxs_freevar, pxs_getstring, pxs_isexception, pxs_isfunction, pxs_isint, pxs_islist, pxs_isobject, pxs_isstring, pxs_new_shallowcopy, pxs_newarena, pxs_newexception, pxs_newhost, pxs_newint, pxs_newmod, pxs_newnull, pxs_newstring, pxs_newtype, pxs_object_addfunc, pxs_object_addprop, pxs_object_callrt, pxs_objectget, pxs_objectset, pxs_var_fromname, pxs_varcall, shared::{module::pxs_ModuleT, pxs_Opaque, pxs_Runtime, var::pxs_VarT},
};

/// Parse the return value of a _call or _get
/// It will determine if a `RuntimeObject` needs to be created.
fn parse_return_value(rt: pxs_Runtime, value: pxs_VarT) -> pxs_VarT {
    if pxs_isfunction(value) {
        RuntimeObject::new(RuntimeObjectType::Function, value, rt)
    } else if pxs_isobject(value) {
        RuntimeObject::new(RuntimeObjectType::Object, value, rt)
    } else {
        value
    }
}

/// @pxs(RuntimeObjectType)
/// @enum
/// @values(Runtime = 0, Function = 1, Object = 2)
/// Determines what the #`RuntimeObject` can and can not do.
///
/// Runtime = [_call, _exec, _eval, _get, _set].
/// Function = [_call]
/// Object = [_call, _get, _set]
#[derive(PartialEq, Clone)]
#[repr(u8)]
enum RuntimeObjectType {
    /// Can do: _call, _exec, _eval, _get.
    Runtime = 0,
    /// Can do: _call.
    Function = 1,
    /// Can do: _call, _get, _set
    Object = 2,
}

/// @pxs(RuntimeObject)
/// A value from another runtime. Each `RuntimeObject` operates differently. See #`RuntimeObjectType` for more details.
struct RuntimeObject {
    _type: RuntimeObjectType,
    value: pxs_VarT,
    runtime: pxs_Runtime,
}

impl PtrMagic for RuntimeObject {}

impl RuntimeObject {
    extern "C" fn free(ptr: pxs_Opaque) {
        if !ptr.is_null() {
            let ro = unsafe { Self::from_raw_void(ptr) };
            if !ro.value.is_null() {
                pxs_freevar(ro.value);
            }
        }
    }

    /// Create a new runtime object. This is never called directly from
    /// user code. Which means that `RuntimeObject` is not exposed as a constructor.
    fn new(_type: RuntimeObjectType, value: pxs_VarT, rt: pxs_Runtime) -> pxs_VarT {
        let rtobj = RuntimeObject {
            _type: _type.clone(),
            value,
            runtime: rt,
        };

        let obj_name = match _type {
            RuntimeObjectType::Runtime => c"RuntimeObjectRuntime".as_ptr(),
            RuntimeObjectType::Function => c"RuntimeObjectFunction".as_ptr(),
            RuntimeObjectType::Object => c"RuntimeObjectObject".as_ptr(),
        };

        let obj = pxs_newtype(
            rtobj.into_void(),
            Self::free,
            obj_name,
            super::PxsCoreType::RuntimeObject as i32,
        );

        // FUNCTIONS
        pxs_object_addfunc(obj, c"_call".as_ptr(), Self::_call);

        if _type != RuntimeObjectType::Function {
            pxs_object_addfunc(obj, c"_get".as_ptr(), Self::_get);
        }
        if _type == RuntimeObjectType::Object {
            pxs_object_addfunc(obj, c"_set".as_ptr(), Self::_set);
        }
        if _type == RuntimeObjectType::Runtime {
            pxs_object_addfunc(obj, c"_exec".as_ptr(), Self::_exec);
            pxs_object_addfunc(obj, c"_eval".as_ptr(), Self::_eval);
        }

        // PROPERTIES
        pxs_object_addprop(obj, c"_type".as_ptr(), Self::_type);

        pxs_newhost(obj)
    }

    /// @pxs
    /// @classmethod(#`RuntimeObject`)
    /// @prop(get) the internal type.
    /// @return(pxs_Int64) the type as a integer.
    /// Get the type value.
    extern "C" fn _type(args: pxs_VarT) -> pxs_VarT {
        let this = get_self!(args, PxsCoreType::RuntimeObject as i32);

        return pxs_newint(this._type.clone() as i64);
    }

    /// @pxs
    /// @self
    /// @except
    /// @classmethod(#`RuntimeObject`)
    /// @args(name:pxs_String|pxs_List(Can be either the name or args if calling itself.), args:pxs_List?(arguments of the function. Is only null when passing args as name.))
    /// @return(#`RuntimeObject`|pxs_Var)
    /// @example(begin)
    /// from pxs import print
    /// from pxs.cross import access
    /// 
    /// lua = access(0)
    /// lua._exec("function add(n1, n2) return n1 + n2 end")
    /// add = lua._get('add')
    /// print(add._call([1,2]))
    /// @example(end)
    /// Depending on the `type`, it will either call a function or call itself.
    /// If `type` is Runtime, it will call a function.
    /// If `type` is Function, it will call itself.
    /// If `type` is Object, it will call a property of itself.
    extern "C" fn _call(args: pxs_VarT) -> pxs_VarT {
        let this = get_self!(args, PxsCoreType::RuntimeObject as i32);

        let name_arg = pxs_arg(args, 1);
        let mut res: pxs_VarT = pxs_newnull();
        let arena = pxs_newarena();
        let runtime_var = pxs_arenaput(arena, this.runtime.into_var().into_raw());
        let runtime_clone = this.runtime.clone();
        // Object or Runtime path.
        if pxs_isstring(name_arg) {
            let call_args_arg = pxs_arg(args, 2);
            if !pxs_islist(call_args_arg) {
                return pxs_newexception(c"Expected args as List.".as_ptr());
            }
            let name = pxs_arena_putstr(arena, pxs_getstring(name_arg));
            let call_args_copy = pxs_new_shallowcopy(call_args_arg);
            // Do the string way.
            res = if this._type == RuntimeObjectType::Object {
                // regular object call
                parse_return_value(
                    runtime_clone,
                    pxs_object_callrt(this.runtime.clone(), this.value, name, call_args_copy),
                )
            } else if this._type == RuntimeObjectType::Runtime {
                parse_return_value(runtime_clone, pxs_call(runtime_var, name, call_args_copy))
            } else {
                unreachable!()
            };
        } else if pxs_islist(name_arg) {
            // This is a function.
            let call_args = pxs_new_shallowcopy(name_arg);
            res = parse_return_value(
                runtime_clone,
                pxs_varcall(runtime_var, this.value, call_args),
            );
        }
        pxs_freearena(arena);

        res
    }

    /// @pxs
    /// @self
    /// @except
    /// @args(name:pxs_String(the name of the variable.))
    /// @return(#`RuntimeObject`|pxs_Var) The result of the get.
    /// @example(begin)
    /// from pxs import print
    /// from pxs.cross import access
    /// js = access(2)
    /// js._exec("""
    ///     globalThis.Me = {
    ///         name: "Jordan"
    ///     };
    /// """)
    /// 
    /// js_me = js._get('Me')
    /// print(js_me._get('name'))
    /// @example(end)
    /// Get a variable/object/function.
    /// If `type` is Runtime, it will get from global scope.
    /// If `type` is Object, it will get a property of itself.
    extern "C" fn _get(args: pxs_VarT) -> pxs_VarT {
        let this = get_self!(args, PxsCoreType::RuntimeObject as i32);
        let name_arg = pxs_arg(args, 1);

        if !pxs_isstring(name_arg) {
            return pxs_newexception(c"Expected String".as_ptr());
        }

        let arena = pxs_newarena();
        let name = pxs_arena_putstr(arena, pxs_getstring(name_arg));
        let runtime_clone = this.runtime.clone();
        let runtime_var = pxs_arenaput(arena, this.runtime.into_var().into_raw());
        let mut res = pxs_newnull();

        if this._type == RuntimeObjectType::Function {
            res = pxs_newexception(c"type must be of Object or Runtime.".as_ptr());
        } else if this._type == RuntimeObjectType::Runtime {
            res = parse_return_value(runtime_clone, pxs_var_fromname(runtime_var, name))
        } else if this._type == RuntimeObjectType::Object {
            res = parse_return_value(runtime_clone, pxs_objectget(runtime_var, this.value, name));
        }

        pxs_freearena(arena);
        res
    }

    /// @pxs
    /// @self
    /// @except
    /// @args(key:pxs_String(the name of the variable.), value:pxs_Var(a value.))
    /// @example(begin)
    /// from pxs import print
    /// from pxs.cross import access
    /// js = access(2)
    /// js._exec("""
    ///     globalThis.Me = {
    ///         name: "Jordan"
    ///     };
    /// """)
    /// 
    /// js_me = js._get('Me')
    /// js_me._set('name', 'Jordan Castro')
    /// print(js_me._get('name'))
    /// @example(end)
    /// Get a variable/object/function.
    /// Only on `type` Object.
    extern "C" fn _set(args: pxs_VarT) -> pxs_VarT {
        let this = get_self!(args, PxsCoreType::RuntimeObject as i32);
        if !pxs_argc(args) == 3 {
            return pxs_newexception(c"Expected 3 arguments (self, key, value)".as_ptr());
        }
        let name_arg = pxs_arg(args, 1);

        if !pxs_isstring(name_arg) {
            return pxs_newexception(c"Expected String".as_ptr());
        }

        let arena = pxs_newarena();
        let runtime_var = pxs_arenaput(arena, this.runtime.into_var().into_raw());
        let name = pxs_arena_putstr(arena, pxs_getstring(name_arg));
        pxs_objectset(
            runtime_var,
            this.value,
            name,
            pxs_new_shallowcopy(pxs_arg(args, 2)),
        );
        pxs_freearena(arena);

        pxs_newnull()
    }

    /// @pxs
    /// @self
    /// @except
    /// @args(code:pxs_String(the code to execute.), file_name:pxs_String(optional name for debugging.))
    /// @return(pxs_String) string if a exception occured.
    /// @example(begin)
    /// from pxs import print
    /// from pxs.cross import access
    /// py = access(1)
    /// py._exec("print('hello!')")
    /// @example(end)
    /// Execute arbitrary code in the runtime.
    /// Only on `type` Runtime.
    extern "C" fn _exec(args: pxs_VarT) -> pxs_VarT {
        let this = get_self!(args, PxsCoreType::RuntimeObject as i32);
        let code_arg = pxs_arg(args, 1);
        if !pxs_isstring(code_arg) {
            return pxs_newexception(c"Expected String".as_ptr());
        }
        let arena = pxs_newarena();

        let name_arg = pxs_arg(args, 2);
        let name: String;
        if !pxs_isstring(name_arg) {
            name = "<pxs_cross_exec>".to_string();
        } else {
            name = own_string!(pxs_getstring(name_arg));
        }

        let mut cstring = CStringSafe::new();
        // Execute
        let mut res = pxs_exec(
            this.runtime.clone(),
            pxs_arena_putstr(arena, pxs_getstring(code_arg)),
            cstring.new_string(&name),
        );
        // If exception, get string.
        if pxs_isexception(res) {
            let msg = pxs_arena_putstr(arena, pxs_getstring(res));
            res = pxs_newstring(msg);
        }
        pxs_freearena(arena);
        res
    }

    /// @pxs
    /// @self
    /// @except
    /// @args(code:pxs_String(the code to evaluate.), file_name:pxs_String(optional name for debugging.))
    /// @return(pxs_Var) The result of the evaluation. If a exception is thrown in the evaluation, it is also thrown in the current runtime and it must be caught.
    /// @example(begin)
    /// from pxs import print
    /// from pxs.cross import access
    /// js = access(2)
    /// result = js._eval('1 + 1')
    /// print(result)
    /// @example(end)
    /// Evaluate arbitrary code in the runtime.
    /// Only on `type` Runtime.
    extern "C" fn _eval(args: pxs_VarT) -> pxs_VarT {
        let this = get_self!(args, PxsCoreType::RuntimeObject as i32);
        let code_arg = pxs_arg(args, 1);
        if !pxs_isstring(code_arg) {
            return pxs_newexception(c"Expected String".as_ptr());
        }
        let name_arg = pxs_arg(args, 2);
        let name: String;
        if !pxs_isstring(name_arg) {
            name = "<pxs_cross_eval>".to_string();
        } else {
            name = own_string!(pxs_getstring(name_arg));
        }

        let arena = pxs_newarena();
        let mut cstring = CStringSafe::new();

        let res = pxs_evalnamed(
            pxs_arena_putstr(arena, pxs_getstring(code_arg)),
            cstring.new_string(&name),
            this.runtime.clone(),
        );
        pxs_freearena(arena);
        res
    }
}

/// @pxs
/// @except
/// @args(rt:pxs_Int64(the runtime value.))
/// @return(#`RuntimeObject`) access to the runtime.
/// @example(begin)
/// from pxs.cross import access
/// from pxs import print
/// lua = access(0)
/// lua._exec(" function add(n1,n2) return n1 + n2 end ")
/// print(lua._call('add', [1,2]))
/// @example(end)
/// Access a pxs runtime via scripting.
extern "C" fn access(args: pxs_VarT) -> pxs_VarT {
    // runtime
    let rt = pxs_arg(args, 0);
    if !pxs_isint(rt) {
        return pxs_newexception(c"Expected integer".as_ptr());
    }

    let rt = unsafe { pxs_Runtime::from_var_ptr(rt) };
    if rt.is_none() {
        return pxs_newexception(c"No runtime found.".as_ptr());
    }

    RuntimeObject::new(RuntimeObjectType::Runtime, pxs_newnull(), rt.unwrap())
}

pub(super) fn init(pxs: pxs_ModuleT) {
    let cross = pxs_newmod(c"cross".as_ptr());

    pxs_addfunc(cross, c"access".as_ptr(), access);

    pxs_add_submod(pxs, cross);
}
