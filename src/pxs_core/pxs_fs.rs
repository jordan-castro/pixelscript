use etffi::{cstring::CStringSafe, own_string};

use crate::{pxs_addfunc, pxs_addmod, pxs_addvar, pxs_arg, pxs_argc, pxs_getint, pxs_getrt, pxs_newbytes, pxs_newexception, pxs_newint, pxs_newmod, pxs_newnull, pxs_newstring, pxs_smart_getstring, shared::{pxs_Opaque, var::pxs_VarT}};

/// How to read a file. Pass in `read_file`.
enum ReadFile {
    Text = 1,
    Bytes = 2
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
                },
                Err(e) => err = e,
            }
        },
        ReadFile::Bytes => {
            let bytes = std::fs::read(path);
            match bytes {
                Ok(mut val) => {
                    // Convert into bytes
                    return pxs_newbytes(val.as_mut_ptr() as pxs_Opaque, size_of::<u8>(), val.len());
                },
                Err(e) => err = e,
            }
        },
    }

    let mut cstring = CStringSafe::new();
    pxs_newexception(cstring.new_string(&err.to_string()))
}

pub(crate) fn init() {
    let pxs_fs = pxs_newmod(c"pxs_fs".as_ptr());

    // Enums
    pxs_addvar(pxs_fs, c"READ_FILE_TEXT".as_ptr(), pxs_newint(1));
    pxs_addvar(pxs_fs, c"READ_FILE_BYTES".as_ptr(), pxs_newint(2));

    // Functions
    pxs_addfunc(pxs_fs, c"read_file".as_ptr(), read_file);

    pxs_addmod(pxs_fs);
}