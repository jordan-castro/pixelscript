#[cfg(feature = "testing")]
use crate::{
    create_raw_string, free_raw_string, own_var, pxs_addfunc, pxs_exec, pxs_freevar, pxs_listget, pxs_listlen, pxs_newmod, pxs_newnull, pxs_tostring, shared::{PtrMagic, func::pxs_Func, module::pxs_Module, pxs_Runtime, var::{pxs_Var, pxs_VarT}}
};

/// A useful macro for debuggin in pixelscript.
#[macro_export]
macro_rules! pxs_debug {
    ($($arg:tt)*) =>
    {
        #[cfg(feature = "pxs-debug")]
        {
            let loc = std::panic::Location::caller();
            eprintln!(
                "[PXS_DEBUG {}:{}] {}",
                loc.file(),
                loc.line(),
                format_args!($($arg)*)
            );
        }
    }
}

#[macro_export]
/// Macro to wrap features
macro_rules! with_feature {
    ($feature:expr, $logic:block) => {
        #[cfg(feature=$feature)]
        {
            $logic
        }
    };
    ($feature:literal, $logic:block, $fallback:block) => {{
        #[cfg(feature = $feature)]
        {
            $logic
        }
        #[cfg(not(feature = $feature))]
        {
            $fallback
        }
    }};
}

#[cfg(feature = "testing")]
pub fn create_module(name: &str) -> *mut pxs_Module {
    let cname = create_raw_string!(name);
    let module = pxs_newmod(cname);
    unsafe {
        free_raw_string!(cname);
    }

    module
}

#[cfg(feature = "testing")]
pub fn add_function(module: *mut pxs_Module, name: &str, function: pxs_Func) {
    let cname = create_raw_string!(name);
    pxs_addfunc(module, cname, function);
    unsafe {
        free_raw_string!(cname);
    }
}

#[cfg(feature = "testing")]
pub fn execute_code(code: &str, file_name: &str, runtime: pxs_Runtime) -> pxs_Var {
    let ccode = create_raw_string!(code);
    let cfile_name = create_raw_string!(file_name);

    let res = pxs_exec(runtime, ccode, cfile_name);

    unsafe {
        free_raw_string!(ccode);
        free_raw_string!(cfile_name);
    }

    own_var!(res)
}

#[cfg(feature = "testing")]
pub extern "C" fn print(args: pxs_VarT) -> pxs_VarT {
    unsafe {
        let runtime = pxs_listget(args, 0);

        let mut string = String::new();
        for i in 1..pxs_listlen(args) {
            let var = pxs_tostring(runtime, pxs_listget(args, i));
            if let Ok(s) = (*var).get_string() {
                string.push_str(format!("{s} ").as_str());
            }
            pxs_freevar(var);
        }

        println!("From Runtime: {string}");
    }

    pxs_newnull()
}

#[cfg(feature = "testing")]
pub fn setup_pxs() {
    let module = create_module("pxs");
    // Add print function
    add_function(module, "print", print);
}