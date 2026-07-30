use crate::{pxs_addmod, pxs_newmod};

/// Initialize `pxs_pxs` module.
pub(crate) fn init() {
    let pxs_pxs = pxs_newmod(c"pxs_pxs".as_ptr());
    pxs_addmod(pxs_pxs);
}