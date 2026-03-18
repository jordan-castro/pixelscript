use crate::{pxs_listget, pxs_varis, shared::var::{pxs_VarT, pxs_VarType}};

// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
pub mod pxs_json;

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