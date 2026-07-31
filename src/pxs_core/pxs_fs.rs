use etffi::{cstring::CStringSafe, own_string, ptr_magic::PtrMagic};

use crate::{
    pxs_addfunc, pxs_addmod, pxs_addvar, pxs_arg, pxs_argc, pxs_copybytes, pxs_getint, pxs_getrt,
    pxs_getstring, pxs_isstring, pxs_newbool, pxs_newbytes, pxs_newexception, pxs_newint,
    pxs_newmod, pxs_newnull, pxs_newstring, pxs_smart_getstring, pxs_varis, pxs_varsize,
    pxs_vartype,
    shared::{
        pxs_Opaque,
        var::{pxs_Var, pxs_VarT, pxs_VarType::pxs_String},
    },
};

/// How to read a file. Pass in `read_file`.
enum ReadFile {
    Text = 1,
    Bytes = 2,
}

/// Read a file, returns either a pxs_String, or pxs_List of pxs_Byte.
extern "C" fn read_file(args: pxs_VarT) -> pxs_VarT {
    // Requires at least 1 arg
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected at least 1 arg".as_ptr());
    }

    let path = own_string!(pxs_smart_getstring(pxs_getrt(args), pxs_arg(args, 0)));
    // Check for read type
    let read_type_num = pxs_getint(pxs_arg(args, 1));
    let read_file_type = if read_type_num == 2 {
        ReadFile::Bytes
    } else {
        ReadFile::Text
    };

    // Catch a error. Just in case.
    let err: std::io::Error;

    // Open the file doh!
    match read_file_type {
        ReadFile::Text => {
            let contents = std::fs::read_to_string(path);
            match contents {
                Ok(val) => {
                    let mut cstring = CStringSafe::new();
                    return pxs_newstring(cstring.new_string(&val));
                }
                Err(e) => err = e,
            }
        }
        ReadFile::Bytes => {
            let bytes = std::fs::read(path);
            match bytes {
                Ok(mut val) => {
                    // Convert into bytes
                    return pxs_newbytes(
                        val.as_mut_ptr() as pxs_Opaque,
                        size_of::<u8>(),
                        val.len(),
                    );
                }
                Err(e) => err = e,
            }
        }
    }

    let mut cstring = CStringSafe::new();
    pxs_newexception(cstring.new_string(&err.to_string()))
}

/// Check if a path exists
extern "C" fn exists(args: pxs_VarT) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    // Get path
    let path = own_string!(pxs_smart_getstring(pxs_getrt(args), pxs_arg(args, 0)));

    let res = std::fs::exists(path);

    match res {
        Ok(val) => pxs_newbool(val),
        Err(err) => {
            let mut cstring = CStringSafe::new();
            pxs_newexception(cstring.new_string(&err.to_string()))
        }
    }
}

/// Check if is a directory.
extern "C" fn is_dir(args: pxs_VarT) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    let path = own_string!(pxs_smart_getstring(pxs_getrt(args), pxs_arg(args, 0)));
    let p = std::path::Path::new(&path);

    return pxs_newbool(p.is_dir());
}

/// Check if is a file
extern "C" fn is_file(args: pxs_VarT) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    let path = own_string!(pxs_smart_getstring(pxs_getrt(args), pxs_arg(args, 0)));
    let p = std::path::Path::new(&path);

    return pxs_newbool(p.is_file());
}

/// Write to a file.
extern "C" fn write_file(args: pxs_VarT) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) != 2 {
        return pxs_newexception(c"Expected 2 args".as_ptr());
    }

    // Check string
    let path_arg = pxs_arg(args, 0);
    if !pxs_varis(path_arg, pxs_String) {
        return pxs_newexception(c"Expected 0 to be String".as_ptr());
    }
    let path = own_string!(pxs_getstring(path_arg));

    // Contents can be string or Bytes. But we dont care. Just convert them into bytes
    let mut bytes = vec![0; pxs_varsize(pxs_arg(args, 1))];
    pxs_copybytes(pxs_arg(args, 1), bytes.as_mut_ptr() as pxs_Opaque);

    // Write bytes
    let res = std::fs::write(path, bytes);
    match res {
        Ok(_) => {}
        Err(err) => {
            let mut cstring = CStringSafe::new();
            return pxs_newexception(cstring.new_string(&err.to_string()));
        }
    }

    pxs_newnull()
}

/// Remove a file
extern "C" fn remove_file(args: pxs_VarT) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    let path_arg = pxs_arg(args, 0);
    if !pxs_isstring(path_arg) {
        return pxs_newexception(c"Expected 0 to be String".as_ptr());
    }
    let path = own_string!(pxs_getstring(path_arg));
    match std::fs::remove_file(path) {
        Ok(_) => {}
        Err(err) => {
            let mut cstring = CStringSafe::new();
            return pxs_newexception(cstring.new_string(&err.to_string()));
        }
    }

    pxs_newnull()
}

/// Create a directory.
extern "C" fn create_dir(args: pxs_VarT) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    let path_arg = pxs_arg(args, 0);
    if !pxs_isstring(path_arg) {
        return pxs_Var::incorrect_type_ep(pxs_String, pxs_vartype(path_arg)).into_raw();
    }
    let path = own_string!(pxs_getstring(path_arg));

    // On error return it!
    match std::fs::create_dir(path) {
        Ok(_) => {}
        Err(err) => {
            let mut cstring = CStringSafe::new();
            return pxs_newexception(cstring.new_string(&err.to_string()));
        }
    }

    pxs_newnull()
}

/// Create a directory recursively
extern "C" fn create_dirs(args: pxs_VarT) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    let path_arg = pxs_arg(args, 0);
    if !pxs_isstring(path_arg) {
        return pxs_Var::incorrect_type_ep(pxs_String, pxs_vartype(path_arg)).into_raw();
    }
    let path = own_string!(pxs_getstring(path_arg));

    // On error return it!
    match std::fs::create_dir_all(path) {
        Ok(_) => {}
        Err(err) => {
            let mut cstring = CStringSafe::new();
            return pxs_newexception(cstring.new_string(&err.to_string()));
        }
    }

    pxs_newnull()
}

/// Remove a empty directory
extern "C" fn remove_empty_dir(args: pxs_VarT) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    let path_arg = pxs_arg(args, 0);
    if !pxs_isstring(path_arg) {
        return pxs_Var::incorrect_type_ep(pxs_String, pxs_vartype(path_arg)).into_raw();
    }
    let path = own_string!(pxs_getstring(path_arg));

    match std::fs::remove_dir(path) {
        Ok(_) => {}
        Err(err) => {
            let mut cstring = CStringSafe::new();
            return pxs_newexception(cstring.new_string(&err.to_string()));
        }
    }

    pxs_newnull()
}

/// Remove a directory, regardless of emptyness.
extern "C" fn remove_dir(args: pxs_VarT) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    let path_arg = pxs_arg(args, 0);
    if !pxs_isstring(path_arg) {
        return pxs_Var::incorrect_type_ep(pxs_String, pxs_vartype(path_arg)).into_raw();
    }
    let path = own_string!(pxs_getstring(path_arg));

    match std::fs::remove_dir_all(path) {
        Ok(_) => {}
        Err(err) => {
            let mut cstring = CStringSafe::new();
            return pxs_newexception(cstring.new_string(&err.to_string()));
        }
    }

    pxs_newnull()
}

pub(crate) fn init() {
    let pxs_fs = pxs_newmod(c"pxs_fs".as_ptr());

    // Enums
    pxs_addvar(pxs_fs, c"READ_FILE_TEXT".as_ptr(), pxs_newint(1));
    pxs_addvar(pxs_fs, c"READ_FILE_BYTES".as_ptr(), pxs_newint(2));

    // Functions
    pxs_addfunc(pxs_fs, c"read_file".as_ptr(), read_file);
    pxs_addfunc(pxs_fs, c"exists".as_ptr(), exists);
    pxs_addfunc(pxs_fs, c"is_dir".as_ptr(), is_dir);
    pxs_addfunc(pxs_fs, c"is_file".as_ptr(), is_file);
    pxs_addfunc(pxs_fs, c"write_file".as_ptr(), write_file);
    pxs_addfunc(pxs_fs, c"remove_file".as_ptr(), remove_file);
    pxs_addfunc(pxs_fs, c"create_dir".as_ptr(), create_dir);
    pxs_addfunc(pxs_fs, c"create_dirs".as_ptr(), create_dirs);
    pxs_addfunc(pxs_fs, c"remove_empty_dir".as_ptr(), remove_empty_dir);
    pxs_addfunc(pxs_fs, c"remove_dir".as_ptr(), remove_dir);

    // Objects

    pxs_addmod(pxs_fs);
}
