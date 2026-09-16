use crate::shared::module::pxs_Module;

unsafe extern "C" {
    /// cbindgen:ignore
    fn pxs_corelib_zip_init();
}

pub(crate) fn init(_module: *mut pxs_Module) {
    unsafe { pxs_corelib_zip_init() };
}