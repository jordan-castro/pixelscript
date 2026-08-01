use etffi::{cstring::CStringSafe, own_string};

use crate::{
    pxs_addfunc, pxs_addmod, pxs_addvar, pxs_arg, pxs_argc, pxs_getstring, pxs_listadd,
    pxs_newexception, pxs_newlist, pxs_newmod, pxs_newnull, pxs_newstring, pxs_varis,
    shared::var::{
        pxs_VarT,
        pxs_VarType::{pxs_String},
    },
};

/// Get the current working directory
extern "C" fn get_cwd(_args: pxs_VarT) -> pxs_VarT {
    let mut cstring = CStringSafe::new();
    let path = std::env::current_dir();
    match path {
        Ok(res) => pxs_newstring(cstring.new_string(res.to_str().unwrap())),
        Err(err) => pxs_newexception(cstring.new_string(&err.to_string())),
    }
}

/// Change the current directory
extern "C" fn chdir(args: pxs_VarT) -> pxs_VarT {
    // Ensure at least 1 arg
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    let new_path_arg = pxs_arg(args, 0);
    // Check if string
    if !pxs_varis(new_path_arg, pxs_String) {
        return pxs_newexception(c"Expected String".as_ptr());
    }

    // Get the string
    let new_path = own_string!(pxs_getstring(new_path_arg));

    // Change the directory
    match std::env::set_current_dir(new_path) {
        Ok(_) => pxs_newnull(),
        Err(err) => {
            let mut cstring = CStringSafe::new();
            pxs_newexception(cstring.new_string(&err.to_string()))
        }
    }
}

/// Initialize `pxs_os` module.
pub(crate) fn init() {
    let pxs_os = pxs_newmod(c"pxs_os".as_ptr());
    let mut cstring = CStringSafe::new();

    // Add args variable
    let p_args = pxs_newlist();
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 0 {
        let mut cstring = CStringSafe::new();
        for arg in args {
            pxs_listadd(p_args, pxs_newstring(cstring.new_string(&arg)));
        }
    }
    pxs_addvar(pxs_os, c"args".as_ptr(), p_args);
    pxs_addvar(pxs_os, c"name".as_ptr(), pxs_newstring(cstring.new_string(std::env::consts::OS)));

    pxs_addfunc(pxs_os, c"get_cwd".as_ptr(), get_cwd);
    pxs_addfunc(pxs_os, c"chdir".as_ptr(), chdir);

    pxs_addmod(pxs_os);
}
