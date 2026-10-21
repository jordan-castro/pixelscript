// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::ops::BitAnd;

use etffi::{create_raw_string, free_raw_string};

use crate::{pxs_addmod, pxs_newmod, pxs_varis, shared::var::{pxs_VarT, pxs_VarType}, with_feature};

mod errors;

#[cfg(feature="pxs_json")]
/// cbindgen:ignore
pub mod pxs_json;
#[cfg(feature="pxs_mem")] 
/// cbindgen:ignore
mod pxs_mem;
#[cfg(feature="pxs_os")]
/// cbindgen:ignore
mod pxs_os;
#[cfg(feature="pxs_pxs")]
/// cbindgen:ignore
mod pxs_pxs;
#[cfg(feature="pxs_fs")]
/// cbindgen:ignore
mod pxs_fs;
#[cfg(feature="pxs_shell")]
/// cbindgen:ignore
mod pxs_shell;
#[cfg(feature="pxs_zip")]
/// cbindgen:ignore
mod pxs_zip;
#[cfg(feature="pxs_http")]
/// cbindgen:ignore
mod http;
#[cfg(feature="pxs_cross")]
/// cbindgen:ignore
mod pxs_cross;

#[macro_export]
macro_rules! get_self {
    ($args:expr, $_type:expr) => {{
        let thisp = crate::pxs_gettype(crate::pxs_getrt($args), crate::pxs_arg($args, 0), $_type);
        if thisp.is_null() {
            return crate::pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(thisp) };
        this
    }};
}

/// cbindgen:no-export
#[repr(i32)]
pub(self) enum PxsCoreType {
    File = 1,
    Logger = 2,
    #[allow(unused)]
    ZipFile = 3,
    Shell = 4,
    ShellOutput = 5,
    ClientResponse = 6,
    Client = 7,
    RuntimeObject = 8
}

#[repr(u16)]
/// Bitflags for modules.
pub enum pxs_ModuleFlag {
    pxs_JSON = 0,
    pxs_MEM = 1 << 0,
    pxs_OS = 1 << 1,
    pxs_PXS = 1 << 2,
    pxs_FS = 1 << 3,
    pxs_SHELL = 1 << 4,
    pxs_ZIP = 1 << 5,
    pxs_HTTP = 1 << 6,
    pxs_CROSS = 1 << 7
}

impl BitAnd for pxs_ModuleFlag {
    type Output = u16;

    fn bitand(self, rhs: Self) -> Self::Output {
        (self as u16) & (rhs as u16)
    }
}

/// This will check if the arguments are valid to be passed into a pxs_Func.
/// This is only used in core functions exposed to lib.
pub(crate) unsafe fn is_valid_pxs_function(rt: pxs_VarT, args: pxs_VarT) -> bool {
    if args.is_null() {
        return false;
    }
    if !pxs_varis(args, pxs_VarType::pxs_List) {
        return false;
    }
    // Check runtime
    if !pxs_varis(rt, pxs_VarType::pxs_Int64) && !pxs_varis(rt, pxs_VarType::pxs_UInt64) {
        return false;
    }

    true
}

/// Setup core modules.
pub(crate) unsafe fn setup_core_modules(modules: u16) {
    let pxs_mod_name = create_raw_string!("pxs");
    let pxs_module = pxs_newmod(pxs_mod_name);
    unsafe {
        free_raw_string!(pxs_mod_name);
    }

    if modules & pxs_ModuleFlag::pxs_MEM as u16 != 0 {
        with_feature!("pxs_mem", {
            pxs_mem::init(pxs_module);
        }, {
            panic!("pxs_mem is not enabled.");
        });
    }
    if modules & pxs_ModuleFlag::pxs_OS as u16 != 0 {
        with_feature!("pxs_os", {
            pxs_os::init(pxs_module);
        }, {
            panic!("pxs_os is not enabled.");
        });
    }
    if modules & pxs_ModuleFlag::pxs_PXS as u16 != 0 {
        with_feature!("pxs_pxs", {
            pxs_pxs::init(pxs_module);
        }, {
            panic!("pxs_pxs is not enabled");
        });
    }
    if modules & pxs_ModuleFlag::pxs_FS as u16 != 0 {
        with_feature!("pxs_fs", {
            pxs_fs::init(pxs_module);
        }, {
            panic!("pxs_fs is not enabled");
        });
    }
    if modules & pxs_ModuleFlag::pxs_SHELL as u16 != 0 {
        with_feature!("pxs_shell", {
            pxs_shell::init(pxs_module);
        }, {
            panic!("pxs_shell is not enabled");
        });
    }
    // if modules & pxs_ModuleFlag::pxs_ZIP as u8 != 0 {
    //     with_feature!("pxs_zip", {
    //         pxs_zip::init(pxs_module);
    //     }, {
    //         panic!("pxs_zip is not enabled");
    //     });
    // }
    if modules & pxs_ModuleFlag::pxs_HTTP as u16 != 0 {
        with_feature!("pxs_http", {
            http::init(pxs_module);
        }, {
            panic!("pxs_http is not enabled.");
        });
    }

    if modules & pxs_ModuleFlag::pxs_CROSS as u16 != 0 {
        with_feature!("pxs_cross", {
            pxs_cross::init(pxs_module);
        }, {
            panic!("pxs_cross is not enabled.");
        })
    }

    pxs_addmod(pxs_module);
}
