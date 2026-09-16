use std::{
    ffi::OsStr,
    process::{Command, Output},
};

use etffi::{cstring::CStringSafe, own_string, ptr_magic::PtrMagic};

use crate::{
    pxs_add_submod, pxs_addfunc, pxs_addobject, pxs_arg, pxs_argc,
    pxs_core::PxsCoreType,
    pxs_freevar, pxs_getrt, pxs_getstring, pxs_gettype, pxs_isstring, pxs_listadd,
    pxs_new_shallowcopy, pxs_newexception, pxs_newhost, pxs_newint, pxs_newlist, pxs_newmod,
    pxs_newnull, pxs_newstring, pxs_newtype, pxs_object_addfunc, pxs_object_addprop,
    shared::{
        module::pxs_Module,
        pxs_Opaque,
        var::{pxs_Var, pxs_VarT},
    },
};

macro_rules! stdwhateva {
    ($args:expr, $method:ident) => {{
        let thisp = pxs_gettype(
            pxs_getrt($args),
            pxs_arg($args, 0),
            PxsCoreType::ShellOutput as i32,
        );
        if thisp.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };
        let output = this.output.$method.clone();
        let as_str = String::from_utf8_lossy(&output);
        let mut cstring = CStringSafe::new();
        pxs_newstring(cstring.new_string(&as_str))
        // pxs_newbytes(output.as_mut_ptr() as pxs_Opaque, size_of::<u8>(), output.len())
    }};
}

/// Just a wrapper around Output
struct ShellOutput {
    output: Output,
}
impl PtrMagic for ShellOutput {}
impl ShellOutput {
    /// @private
    extern "C" fn free_output(ptr: pxs_Opaque) {
        if !ptr.is_null() {
            let _ = unsafe { ShellOutput::from_raw_void(ptr) };
        }
    }

    /// @private
    fn into_pxs(out: ShellOutput) -> pxs_VarT {
        let obj = pxs_newtype(
            out.into_void(),
            Self::free_output,
            c"ShellOutput".as_ptr(),
            PxsCoreType::ShellOutput as i32,
        );

        // Properties
        pxs_object_addprop(obj, c"stdout".as_ptr(), Self::stdout);
        pxs_object_addprop(obj, c"stderr".as_ptr(), Self::stderr);
        pxs_object_addprop(obj, c"status".as_ptr(), Self::status);

        pxs_newhost(obj)
    }

    /// @self
    /// @prop.get
    /// Get stdout
    /// returns `string`
    extern "C" fn stdout(args: pxs_VarT) -> pxs_VarT {
        stdwhateva!(args, stdout)
    }

    /// @self
    /// @prop.get
    /// Get stderr
    /// returns `string`
    extern "C" fn stderr(args: pxs_VarT) -> pxs_VarT {
        stdwhateva!(args, stderr)
    }

    /// @self
    /// @prop.get
    /// Get status
    /// returns `int`
    extern "C" fn status(args: pxs_VarT) -> pxs_VarT {
        let thisp = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::ShellOutput as i32,
        );
        if thisp.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };
        let status = this.output.status.code().unwrap_or(0);
        pxs_newint(status as i64)
    }
}

/// Just a wrapper around Command.
struct Shell {
    cmd: Command,
}
impl PtrMagic for Shell {}
impl Shell {
    /// @private
    extern "C" fn free_shell(ptr: pxs_Opaque) {
        if !ptr.is_null() {
            let _ = unsafe { Shell::from_raw_void(ptr) };
        }
    }

    /// Create a new platform shell.
    ///
    /// returns `Shell`
    extern "C" fn platform(args: pxs_VarT) -> pxs_VarT {
        let nargs = pxs_newlist();
        pxs_listadd(nargs, pxs_new_shallowcopy(pxs_getrt(args)));

        if cfg!(target_os = "windows") {
            pxs_listadd(nargs, pxs_newstring(c"cmd".as_ptr()));
        } else {
            pxs_listadd(nargs, pxs_newstring(c"sh".as_ptr()));
        }

        let shell = Self::new(nargs);
        pxs_freevar(nargs);
        shell
    }

    /// Create a new shell.
    ///
    /// args:
    ///   - program: `string` the starting program.
    ///
    /// returns `Shell` instance.
    extern "C" fn new(args: pxs_VarT) -> pxs_VarT {
        let program_arg = pxs_arg(args, 0);
        if !pxs_isstring(program_arg) {
            return pxs_newexception(c"Expected string".as_ptr());
        }
        let cmd = Command::new(own_string!(pxs_getstring(program_arg)));
        let shell = Shell { cmd }.into_void();

        let obj = pxs_newtype(
            shell,
            Self::free_shell,
            c"Shell".as_ptr(),
            PxsCoreType::Shell as i32,
        );

        // Functions
        pxs_object_addfunc(obj, c"arg".as_ptr(), Self::arg);
        pxs_object_addfunc(obj, c"status".as_ptr(), Self::status);
        pxs_object_addfunc(obj, c"spawn".as_ptr(), Self::spawn);
        pxs_object_addfunc(obj, c"command".as_ptr(), Self::command);
        pxs_object_addfunc(obj, c"output".as_ptr(), Self::output);

        // Properties

        pxs_newhost(obj)
    }

    /// @except
    /// @self
    /// Define this as a command. Only call this once before adding any args after program.
    extern "C" fn command(args: pxs_VarT) -> pxs_VarT {
        let thisp = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Shell as i32);
        if thisp.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };

        // Check that only 1 arg has been applied
        let args_len: Vec<&OsStr> = this.cmd.get_args().collect::<_>();
        if args_len.len() != 0 {
            return pxs_newexception(c"This must be the first argment".as_ptr());
        }

        if cfg!(target_os = "windows") {
            this.cmd.arg("/C");
        } else {
            this.cmd.arg("-c");
        }

        pxs_newnull()
    }

    /// @self
    /// @except
    /// Add a argument to the command.
    /// args:
    ///   - argument: `string` the argument.
    ///
    extern "C" fn arg(args: pxs_VarT) -> pxs_VarT {
        let thisp = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Shell as i32);
        if thisp.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };
        // Get arg
        let arg = pxs_arg(args, 1);
        if !pxs_isstring(arg) {
            return pxs_newexception(c"Expected String".as_ptr());
        }
        let arg = own_string!(pxs_getstring(arg));
        // Set it
        this.cmd.arg(arg);

        pxs_newnull()
    }

    /// @self
    /// @except
    /// Run and get status code.
    ///
    /// returns `int`
    extern "C" fn status(args: pxs_VarT) -> pxs_VarT {
        let thisp = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Shell as i32);
        if thisp.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };

        match this.cmd.status() {
            Ok(val) => pxs_newint(val.code().unwrap_or(0) as i64),
            Err(err) => pxs_Var::new_exception(err).into_raw(),
        }
    }

    /// @self
    /// @except
    /// Spawn a process.
    extern "C" fn spawn(args: pxs_VarT) -> pxs_VarT {
        let thisp = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Shell as i32);
        if thisp.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };

        match this.cmd.spawn() {
            Ok(_) => pxs_newnull(),
            Err(err) => pxs_Var::new_exception(err).into_raw(),
        }
    }

    /// @self
    /// @except
    /// Run and get output as a `Output`.
    ///
    /// returns `Output` the output
    extern "C" fn output(args: pxs_VarT) -> pxs_VarT {
        let thisp = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Shell as i32);
        if thisp.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };

        match this.cmd.output() {
            Ok(val) => {
                let so = ShellOutput { output: val };
                ShellOutput::into_pxs(so)
            }
            Err(err) => pxs_Var::new_exception(err).into_raw(),
        }
    }
}

/// @except
/// Run a command. Prints to terminal by default.
/// args:
///   - command: `string` the command to run.
///
/// returns `int` the status code.
extern "C" fn system(args: pxs_VarT) -> pxs_VarT {
    if pxs_argc(args) != 1 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }
    let command_str = pxs_arg(args, 0);
    if !pxs_isstring(command_str) {
        return pxs_newexception(c"Expected String".as_ptr());
    }
    let command = own_string!(pxs_getstring(command_str));

    // Check target
    let (program, f) = if cfg!(target_os = "windows") {
        ("cmd".to_string(), "/C")
    } else {
        ("sh".to_string(), "-c")
    };
    let mut cmd = Command::new(program);
    cmd.arg(f);
    cmd.arg(command);

    let status = cmd.status();
    match status {
        Ok(val) => pxs_newint(val.code().unwrap_or(1) as i64),
        Err(err) => pxs_Var::new_exception(err).into_raw(),
    }
}

pub(crate) fn init(module: *mut pxs_Module) {
    let pxs_shell = pxs_newmod(c"shell".as_ptr());

    // Functions
    pxs_addfunc(pxs_shell, c"system".as_ptr(), system);

    // Objects
    pxs_addobject(pxs_shell, c"Shell".as_ptr(), Shell::new);
    pxs_addobject(pxs_shell, c"PlatformShell".as_ptr(), Shell::platform);

    pxs_add_submod(module, pxs_shell);
}
