use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

use etffi::{cstring::CStringSafe, own_string, ptr_magic::PtrMagic};

use crate::{
    pxs_addfunc, pxs_addmod, pxs_addobject, pxs_addvar, pxs_arenaput, pxs_arg, pxs_argc,
    pxs_copybytes,
    pxs_core::PxsCoreType,
    pxs_freearena, pxs_getint, pxs_getrt, pxs_gettype, pxs_listadd, pxs_newarena, pxs_newbool,
    pxs_newbytes, pxs_newcopy, pxs_newexception, pxs_newhost, pxs_newint,
    pxs_newlist, pxs_newmod, pxs_newnull, pxs_newstring, pxs_newtype, pxs_object_addfunc,
    pxs_object_addprop, pxs_smart_getstring, pxs_varsize,
    shared::{
        func::pxs_Func,
        pxs_Opaque,
        var::{pxs_Var, pxs_VarT},
    },
};

/// How to read a file. Pass in `read_file`.
enum ReadFile {
    Text = 1,
    Bytes = 2,
}

/// How to open a file.
enum OpenType {
    Read = 1 << 0,
    Write = 1 << 1,
    Append = 1 << 2,
}

/// Our pxs exposed File object
struct File {
    /// The actual file.
    _f: std::fs::File,
    /// The open type
    open_file: u8,
    /// Path
    path: String,
}

// A helper for working with pointers.
impl PtrMagic for File {}
impl File {
    /// Free a `File`
    extern "C" fn free(ptr: pxs_Opaque) {
        if !ptr.is_null() {
            let _ = unsafe { File::from_raw_void(ptr) };
        }
    }

    /// Open or create a new file.
    /// OpenFile is defaulted to Read if not passed.
    extern "C" fn open(args: pxs_VarT) -> pxs_VarT {
        if pxs_argc(args) == 0 {
            return pxs_newexception(c"Expected 1 arg".as_ptr());
        }

        let path = own_string!(pxs_smart_getstring(pxs_getrt(args), pxs_arg(args, 0)));
        // Check open type
        let ot_val = pxs_getint(pxs_arg(args, 1));
        let mut open_options = OpenOptions::new();
        let mut open_file: u8 = 0;
        if ot_val == -1 {
            open_options.read(true);
            open_file = OpenType::Read as u8;
        } else {
            if (ot_val as u8) & (OpenType::Read as u8) != 0 {
                open_options.read(true);
                open_file |= OpenType::Read as u8;
            }
            if (ot_val as u8) & (OpenType::Write as u8) != 0 {
                open_options.write(true);
                open_file |= OpenType::Write as u8;
                open_options.create(true);
            }
            if (ot_val as u8) & (OpenType::Append as u8) != 0 {
                open_options.append(true);
                open_file |= OpenType::Append as u8;
                open_options.create(true);
            }
        }

        // Open now
        let file = open_options.open(&path);
        match file {
            Ok(val) => {
                let f = File {
                    _f: val,
                    open_file,
                    path,
                };

                // Create a object
                let obj = pxs_newtype(
                    f.into_void(),
                    File::free,
                    c"File".as_ptr(),
                    PxsCoreType::File as i32,
                );
                // Methods
                pxs_object_addfunc(obj, c"write".as_ptr(), File::write);
                pxs_object_addfunc(obj, c"read".as_ptr(), File::read);
                // Properties
                pxs_object_addprop(obj, c"path".as_ptr(), File::path_prop);
                pxs_object_addprop(obj, c"open_type".as_ptr(), File::open_file_prop);

                pxs_newhost(obj)
            }
            Err(err) => pxs_Var::new_exception(err).into_raw(),
        }
    }

    /// Write
    extern "C" fn write(args: pxs_VarT) -> pxs_VarT {
        // Check for this
        let thisp = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::File as i32);
        if thisp.is_null() {
            return pxs_newexception(c"Self expected".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };

        // Check open type
        if this.open_file & (OpenType::Write as u8) == 0
            && this.open_file & (OpenType::Append as u8) == 0
        {
            return pxs_newexception(c"File was not opened to Write.".as_ptr());
        }

        // Copy bytes and write them.
        let mut contents = vec![0; pxs_varsize(pxs_arg(args, 1))];
        pxs_copybytes(pxs_arg(args, 1), contents.as_mut_ptr() as pxs_Opaque);
        match this._f.write(contents.as_slice()) {
            Ok(_) => {}
            Err(err) => {
                return pxs_Var::new_exception(err).into_raw();
            }
        }

        pxs_newnull()
    }

    /// Read
    /// TODO(jc) return num bytes read, make it possible to read up to a certain number at a time.
    extern "C" fn read(args: pxs_VarT) -> pxs_VarT {
        // Check for this
        let thisp = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::File as i32);
        if thisp.is_null() {
            return pxs_newexception(c"Self expected".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };

        // Get read type
        let read_type = match pxs_getint(pxs_arg(args, 1)) {
            2 => ReadFile::Bytes,
            _ => ReadFile::Text,
        };

        let mut buffer = vec![];
        // let mut buffer = vec![0;len as usize];
        match this._f.read_to_end(&mut buffer) {
            Ok(_) => {}
            Err(err) => {
                return pxs_Var::new_exception(err).into_raw();
            }
        }
        match read_type {
            ReadFile::Text => {
                let mut cstring = CStringSafe::new();
                match String::from_utf8(buffer) {
                    Ok(val) => pxs_newstring(cstring.new_string(&val)),
                    Err(err) => pxs_Var::new_exception(err).into_raw(),
                }
            }
            ReadFile::Bytes => pxs_newbytes(
                buffer.as_mut_ptr() as pxs_Opaque,
                size_of::<u8>(),
                buffer.len(),
            ),
        }
    }

    /// Path prop
    extern "C" fn path_prop(args: pxs_VarT) -> pxs_VarT {
        // Check for this
        let thisp = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::File as i32);
        if thisp.is_null() {
            return pxs_newexception(c"Self expected".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };

        let mut cstring = CStringSafe::new();
        pxs_newstring(cstring.new_string(&this.path))
    }

    /// Open type prop
    extern "C" fn open_file_prop(args: pxs_VarT) -> pxs_VarT {
        // Check for this
        let thisp = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::File as i32);
        if thisp.is_null() {
            return pxs_newexception(c"Self expected".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };
        pxs_newint(this.open_file as i64)
    }
}

/// Internal code for handling (create_dir, create_dirs, remove_dir, remove_dirs, remove_file, is_file, is_dir)
fn _internal_runner(args: pxs_VarT, method: fn(String) -> Result<(), std::io::Error>) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    let path = own_string!(pxs_smart_getstring(pxs_getrt(args), pxs_arg(args, 0)));

    // On error return it!
    match method(path) {
        Ok(_) => {}
        Err(err) => {
            let mut cstring = CStringSafe::new();
            return pxs_newexception(cstring.new_string(&err.to_string()));
        }
    }

    pxs_newnull()
}

/// Internal code for handling (is_file, is_dir)
fn _is(args: pxs_VarT, is_file: bool, is_dir: bool) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    let path = own_string!(pxs_smart_getstring(pxs_getrt(args), pxs_arg(args, 0)));
    let p = std::path::Path::new(&path);

    if is_file {
        pxs_newbool(p.is_file())
    } else if is_dir {
        pxs_newbool(p.is_dir())
    } else {
        unreachable!("_is needs true or true dog.")
    }
}

/// Use the internal `File` object for calling a function.
/// Values are handled.
fn _call(
    runtime: pxs_VarT,
    path: pxs_VarT,
    open_type: OpenType,
    function: pxs_Func,
    func_args: Vec<pxs_VarT>,
) -> pxs_VarT {
    let arena = pxs_newarena();
    let open_args = pxs_arenaput(arena, pxs_newlist());
    pxs_listadd(open_args, pxs_newcopy(runtime)); // runtime
    pxs_listadd(open_args, pxs_newcopy(path)); // path
    pxs_listadd(open_args, pxs_newint(open_type as i64));

    let file = File::open(open_args);

    // Setup func args
    let args = pxs_arenaput(arena, pxs_newlist());
    pxs_listadd(args, pxs_newcopy(runtime));
    pxs_listadd(args, file);
    for a in func_args {
        pxs_listadd(args, a);
    }

    let result = unsafe { function(args) };

    pxs_freearena(arena);

    result
}

/// Read a file, returns either a pxs_String, or pxs_List of pxs_Byte.
extern "C" fn read_file(args: pxs_VarT) -> pxs_VarT {
    _call(
        pxs_getrt(args),
        pxs_arg(args, 0),
        OpenType::Read,
        File::read,
        vec![pxs_newcopy(pxs_arg(args, 1))],
    )
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
    _is(args, false, true)
}

/// Check if is a file
extern "C" fn is_file(args: pxs_VarT) -> pxs_VarT {
    _is(args, true, false)
}

/// Write to a file.
extern "C" fn write_file(args: pxs_VarT) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) != 2 {
        return pxs_newexception(c"Expected 2 args".as_ptr());
    }

    _call(
        pxs_getrt(args),
        pxs_arg(args, 0),
        OpenType::Write,
        File::write,
        vec![pxs_newcopy(pxs_arg(args, 1))],
    )
}

/// Append to a file.
extern "C" fn append_file(args: pxs_VarT) -> pxs_VarT {
    // Check argc
    if pxs_argc(args) != 2 {
        return pxs_newexception(c"Expected 2 args".as_ptr());
    }

    _call(
        pxs_getrt(args),
        pxs_arg(args, 0),
        OpenType::Append,
        File::write,
        vec![pxs_newcopy(pxs_arg(args, 1))],
    )
}

/// Remove a file
extern "C" fn remove_file(args: pxs_VarT) -> pxs_VarT {
    _internal_runner(args, std::fs::remove_file)
}

/// Create a directory.
extern "C" fn create_dir(args: pxs_VarT) -> pxs_VarT {
    _internal_runner(args, std::fs::create_dir)
}

/// Create a directory recursively
extern "C" fn create_dirs(args: pxs_VarT) -> pxs_VarT {
    _internal_runner(args, std::fs::create_dir_all)
}

/// Remove a empty directory
extern "C" fn remove_empty_dir(args: pxs_VarT) -> pxs_VarT {
    _internal_runner(args, std::fs::remove_dir)
}

/// Remove a directory, regardless of emptyness.
extern "C" fn remove_dir(args: pxs_VarT) -> pxs_VarT {
    _internal_runner(args, std::fs::remove_dir_all)
}

/// Read contents of directory.
extern "C" fn read_dir(args: pxs_VarT) -> pxs_VarT {
    if pxs_argc(args) == 0 {
        return pxs_newexception(c"Expected 1 arg".as_ptr());
    }

    let path = own_string!(pxs_smart_getstring(pxs_getrt(args), pxs_arg(args, 0)));
    let iter = std::fs::read_dir(path);
    // Get values
    match iter {
        Ok(val) => {
            let res = pxs_newlist();
            let mut cstring = CStringSafe::new();
            for item in val {
                if item.is_err() {
                    continue;
                }
                let i = item.unwrap().path();
                let p = i.to_str().unwrap();
                pxs_listadd(res, pxs_newstring(cstring.new_string(p)));
            }

            res
        }
        Err(err) => pxs_Var::new_exception(err).into_raw(),
    }
}

pub(crate) fn init() {
    let pxs_fs = pxs_newmod(c"pxs_fs".as_ptr());

    // Enums
    pxs_addvar(pxs_fs, c"READ_FILE_TEXT".as_ptr(), pxs_newint(1));
    pxs_addvar(pxs_fs, c"READ_FILE_BYTES".as_ptr(), pxs_newint(2));
    pxs_addvar(
        pxs_fs,
        c"OPEN_TYPE_READ".as_ptr(),
        pxs_newint(OpenType::Read as i64),
    );
    pxs_addvar(
        pxs_fs,
        c"OPEN_TYPE_WRITE".as_ptr(),
        pxs_newint(OpenType::Write as i64),
    );
    pxs_addvar(
        pxs_fs,
        c"OPEN_TYPE_APPEND".as_ptr(),
        pxs_newint(OpenType::Append as i64),
    );

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
    pxs_addfunc(pxs_fs, c"read_dir".as_ptr(), read_dir);
    pxs_addfunc(pxs_fs, c"append_file".as_ptr(), append_file);

    // Objects
    pxs_addobject(pxs_fs, c"File".as_ptr(), File::open);

    // Factories

    pxs_addmod(pxs_fs);
}
