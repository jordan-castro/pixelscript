use crate::{
    borrow_var, own_var, pxs_addfunc, pxs_addmod, pxs_arenaput, pxs_debug, pxs_freearena,
    pxs_listadd, pxs_listget, pxs_listlen, pxs_new_shallowcopy, pxs_newarena, pxs_newcopy,
    pxs_newlist, pxs_newmod, pxs_newnull, pxs_objectget,
    shared::{
        PXS_PTR_NAME, PtrMagic,
        object::apply_ref_count_delete,
        utils::CStringSafe,
        var::{pxs_Var, pxs_VarT, pxs_VarType},
    },
};

/// Delete a `pxs_Var` `PixelObject`.
extern "C" fn pxs_mem_delete(args: pxs_VarT) -> pxs_VarT {
    // Check length is 2 only (RT, object)
    let len = pxs_listlen(args);
    if len != 2 {
        return pxs_Var::expected_n_args_ep(2, len as u32).into_raw();
    }

    // Get idx
    let mut cstrgen = CStringSafe::new();
    let obj_idx = own_var!(pxs_objectget(
        pxs_listget(args, 0),
        pxs_listget(args, 1),
        cstrgen.new_string(PXS_PTR_NAME)
    ));

    if !obj_idx.is_i64() {
        pxs_debug!("{:#?}\n{:#?}", obj_idx, borrow_var!(pxs_listget(args, 1)));
        return pxs_Var::incorrect_type_ep(pxs_VarType::pxs_Int64, obj_idx.tag).into_raw();
    }

    // Drop it
    let idx = obj_idx.get_i64().unwrap();
    apply_ref_count_delete(idx as i32);

    pxs_newnull()
    // pxs_newbool(true)
}

/// Delete a List of `pxs_Var` `PixelObject`
extern "C" fn pxs_mem_delete_all(args: pxs_VarT) -> pxs_VarT {
    let len = pxs_listlen(args);
    if len != 2 {
        return pxs_Var::expected_n_args_ep(2, len as u32).into_raw();
    }

    let list = pxs_listget(args, 1);
    let bvar = borrow_var!(list);
    if !bvar.is_list() {
        return pxs_Var::incorrect_type_ep(pxs_VarType::pxs_List, bvar.tag).into_raw();
    }

    let rt = pxs_listget(args, 0);

    let arena = pxs_newarena();
    for i in 0..pxs_listlen(list) {
        let func_args = pxs_arenaput(arena, pxs_newlist());
        pxs_listadd(func_args, pxs_newcopy(rt));
        pxs_listadd(func_args, pxs_new_shallowcopy(pxs_listget(list, i)));
        // Call mem delete
        pxs_arenaput(arena, pxs_mem_delete(func_args));
    }
    pxs_freearena(arena);

    pxs_newnull()
}

/// Initialize `pxs_mem` module.
pub(crate) fn init() {
    let mut cstrgen = CStringSafe::new();

    let pxs_mem = pxs_newmod(cstrgen.new_string("pxs_mem"));
    pxs_addfunc(pxs_mem, cstrgen.new_string("memdel"), pxs_mem_delete);
    pxs_addfunc(
        pxs_mem,
        cstrgen.new_string("mem_delall"),
        pxs_mem_delete_all,
    );
    pxs_addmod(pxs_mem);
}
