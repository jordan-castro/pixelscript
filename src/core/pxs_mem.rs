use crate::{own_var, pxs_addfunc, pxs_addmod, pxs_listget, pxs_listlen, pxs_newbool, pxs_newmod, pxs_objectget, shared::{PXS_PTR_NAME, PtrMagic, object::apply_ref_count_delete, utils::CStringSafe, var::{pxs_Var, pxs_VarT, pxs_VarType}}};

/// Delete a `pxs_Var` `PixelObject`.
pub extern "C" fn pxs_mem_delete(args: pxs_VarT) -> pxs_VarT {
    // Check length is 2 only (RT, object)
    let len = pxs_listlen(args);
    if len != 2 {
        return pxs_Var::expected_n_args_ep(2, len as u32).into_raw();
    }

    // Get idx
    let mut cstrgen = CStringSafe::new();
    let obj_idx = own_var!(pxs_objectget(pxs_listget(args, 0), pxs_listget(args, 1), cstrgen.new_string(PXS_PTR_NAME)));

    if !obj_idx.is_i64() {
        return pxs_Var::incorrect_type_ep(pxs_VarType::pxs_Int64, obj_idx.tag).into_raw();
    }

    // Drop it
    let idx = obj_idx.get_i64().unwrap();
    apply_ref_count_delete(idx as i32);

    pxs_newbool(true)
}

/// Initialize `pxs_mem` module.
pub(crate) fn init() {
    let mut cstrgen = CStringSafe::new();

    let pxs_mem = pxs_newmod(cstrgen.new_string("pxs_mem"));
    pxs_addfunc(pxs_mem, cstrgen.new_string("memdel"), pxs_mem_delete);
    pxs_addmod(pxs_mem);
}