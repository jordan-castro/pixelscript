unsafe extern "C" {
    fn pxs_corelib_zip_init();
}

pub(crate) fn init() {
    unsafe { pxs_corelib_zip_init() };
}