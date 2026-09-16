use etffi::{cstring::CStringSafe, own_string, ptr_magic::PtrMagic};

use crate::{
    pxs_addfunc, pxs_addobject, pxs_arenaput, pxs_arg, pxs_argc,
    pxs_core::PxsCoreType,
    pxs_freearena, pxs_getbool, pxs_getrt, pxs_getstring, pxs_gettype, pxs_isbool, pxs_isobject,
    pxs_isstring, pxs_listadd, pxs_new_shallowcopy, pxs_newarena, pxs_newbool, pxs_newcopy,
    pxs_newexception, pxs_newhost, pxs_newlist, pxs_newnull, pxs_newstring,
    pxs_newtype, pxs_object_addfunc, pxs_object_addprop, pxs_smart_getstring,
    shared::{module::pxs_Module, pxs_Opaque, var::pxs_VarT},
};

struct Logger {
    seperator: String,
    end_line: bool,
}

impl PtrMagic for Logger {}
impl Logger {
    /// @private
    extern "C" fn free(ptr: pxs_Opaque) {
        if !ptr.is_null() {
            let _ = unsafe { Logger::from_raw_void(ptr) };
        }
    }

    /// Create a new logger
    /// args:
    ///   - sep: `string=" "` optinal seperator.
    ///   - end_line: `bool=true` optional to print a \n.
    ///
    /// returns `Logger` instance.
    extern "C" fn new(args: pxs_VarT) -> pxs_VarT {
        // Get seperator if any
        let sep = if pxs_isstring(pxs_arg(args, 0)) {
            own_string!(pxs_getstring(pxs_arg(args, 0)))
        } else {
            " ".to_string()
        };
        // Check if a end line
        let end_line = if pxs_isbool(pxs_arg(args, 1)) {
            pxs_getbool(pxs_arg(args, 1))
        } else {
            true
        };

        // Setup dog
        let logger = Logger {
            seperator: sep,
            end_line,
        }
        .into_void();
        let obj = pxs_newtype(
            logger,
            Logger::free,
            c"Logger".as_ptr(),
            PxsCoreType::Logger as i32,
        );

        // Methods
        pxs_object_addfunc(obj, c"print".as_ptr(), Logger::print);
        // Props
        pxs_object_addprop(obj, c"seperator".as_ptr(), Logger::seperator_prop);
        pxs_object_addprop(obj, c"end_line".as_ptr(), Logger::end_line_prop);

        pxs_newhost(obj)
    }

    /// @except
    /// @self
    /// Handle print.
    /// args:
    ///   - args: `...` N number of params.
    extern "C" fn print(args: pxs_VarT) -> pxs_VarT {
        // Get this
        let thisp = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Logger as i32,
        );
        if thisp.is_null() {
            return pxs_newexception(c"Self required".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };

        // args
        let mut msg = String::new();
        let rt = pxs_getrt(args);
        let argc = pxs_argc(args);
        for i in 1..argc {
            let arg = pxs_arg(args, i as i32);
            // Skip 0 since it is `this`.
            let contents = own_string!(pxs_smart_getstring(rt, arg));
            msg.push_str(&contents);
            if i < argc - 1 {
                msg.push_str(&this.seperator);
            }
        }

        if this.end_line {
            msg.push_str("\n");
        }

        // Print
        print!("{msg}");

        pxs_newnull()
    }

    /// @except
    /// @self
    /// @prop
    /// seperator prop
    ///
    /// args:
    ///   - new_seperator: `string`
    ///
    /// returns `string`
    extern "C" fn seperator_prop(args: pxs_VarT) -> pxs_VarT {
        let thisp = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Logger as i32,
        );
        if thisp.is_null() {
            return pxs_newexception(c"Self required".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };

        if pxs_argc(args) == 1 {
            // Get
            let mut cstring = CStringSafe::new();
            pxs_newstring(cstring.new_string(&this.seperator))
        } else {
            // Set
            let arg = pxs_arg(args, 1);
            if !pxs_isstring(arg) {
                return pxs_newexception(c"Expected String".as_ptr());
            }

            let new_sep = own_string!(pxs_getstring(arg));
            this.seperator = new_sep;

            pxs_newnull()
        }
    }

    /// @except
    /// @self
    /// @prop
    /// end_line prop
    /// args:
    ///   - end_line: `bool` new end_line
    ///
    /// returns `bool`
    extern "C" fn end_line_prop(args: pxs_VarT) -> pxs_VarT {
        let thisp = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Logger as i32,
        );
        if thisp.is_null() {
            return pxs_newexception(c"Self required".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };

        if pxs_argc(args) == 1 {
            // Get
            pxs_newbool(this.end_line)
        } else {
            // Set
            let arg = pxs_arg(args, 1);
            if !pxs_isbool(arg) {
                return pxs_newexception(c"Expected bool".as_ptr());
            }

            let n = pxs_getbool(arg);
            this.end_line = n;

            pxs_newnull()
        }
    }
}

/// Print to stdout
/// args:
///   - args: `...` n number of args to print. Pass in `Logger` instance to override default.
extern "C" fn print(args: pxs_VarT) -> pxs_VarT {
    let rt = pxs_getrt(args);
    let arena = pxs_newarena();
    let mut skip_0 = false;
    // Check for logger
    let log_ptr = {
        // Make sure its actually a object.
        let arg = pxs_arg(args, 0);
        let pos = if pxs_isobject(arg) {
            // Get pointer
            pxs_gettype(rt, arg, PxsCoreType::Logger as i32)
        } else {
            core::ptr::null_mut()
        };
        if pos.is_null() {
            // Create one
            let nargs = pxs_arenaput(arena, pxs_newlist());
            pxs_listadd(nargs, pxs_newcopy(rt));
            Logger::new(nargs)
            // res
        } else {
            skip_0 = true;
            pxs_arg(args, 0)
        }
    };

    let nargs = pxs_arenaput(arena, pxs_newlist());
    pxs_listadd(nargs, pxs_newcopy(rt));
    if skip_0 {
        pxs_listadd(nargs, pxs_new_shallowcopy(log_ptr));
    } else {
        pxs_listadd(nargs, log_ptr);
    }

    for i in 0..pxs_argc(args) {
        if i == 0 && skip_0 {
            continue;
        }
        pxs_listadd(nargs, pxs_new_shallowcopy(pxs_arg(args, i as i32)));
    }

    // Now pass it to be called.
    let res = Logger::print(nargs);
    pxs_freearena(arena);
    res
}

/// @except
/// Error if expression is not true
/// args:
///   - expr: `bool` the expression being evaluated.
///   - msg: `string` if failed error this.
extern "C" fn passert(args: pxs_VarT) -> pxs_VarT {
    let expr = pxs_arg(args, 0);
    if !pxs_isbool(expr) {
        return pxs_newexception(c"Expected boolean".as_ptr());
    }
    let msg = pxs_arg(args, 1);
    if !pxs_isstring(msg) {
        return pxs_newexception(c"Expected String".as_ptr());
    }
    let msg: String = own_string!(pxs_getstring(msg));

    if !pxs_getbool(expr) {
        let mut cstring = CStringSafe::new();
        return pxs_newexception(cstring.new_string(&msg));
    }

    pxs_newnull()
}

/// @private
/// Add `pxs` module functions and objects.
/// Without this, it will not be possible to print or assert using the pxs native.
pub(crate) fn init(module: *mut pxs_Module) {
    // Methods
    pxs_addfunc(module, c"print".as_ptr(), print);
    pxs_addfunc(module, c"passert".as_ptr(), passert);

    // Objects
    pxs_addobject(module, c"Logger".as_ptr(), Logger::new);
}
