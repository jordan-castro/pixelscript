use crate::{pxs_addmod, pxs_newmod};

pub(crate) fn init() {
    let pxs_shell = pxs_newmod(c"pxs_shell".as_ptr());
    pxs_addmod(pxs_shell);
}