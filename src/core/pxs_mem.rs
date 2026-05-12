// use crate::{borrow_var, pxs_addfunc, pxs_addmod, pxs_newbool, pxs_newmod, shared::{PtrMagic, utils::CStringSafe, var::{pxs_Var, pxs_VarT}}};

// /// Delete a `pxs_Var` `PixelObject`.
// pub extern "C" fn pxs_mem_delete(args: pxs_VarT) -> pxs_VarT {
//     let list = borrow_var!(args).get_list().unwrap();

//     // I.E. only RT or more than 1 object
//     if list.len() == 1 || list.len() > 2 {
//         // requires ONLY one variable
//         return pxs_newbool(false);
//     }

//     // Get the pointer
//     let obj_var = list.get_item(1).unwrap();
//     let obj = obj_var.

//     pxs_newbool(true)
// }

// /// Initialize `pxs_mem` module.
// pub(crate) fn init() {
//     let mut cstrgen = CStringSafe::new();

//     let pxs_mem = pxs_newmod(cstrgen.new_string("pxs_mem"));
//     pxs_addfunc(pxs_mem, cstrgen.new_string("memdel"), pxs_mem_delete);
//     pxs_addmod(pxs_mem);
// }