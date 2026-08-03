unsafe extern "C" {
    /// cbindgen:ignore
    fn pxs_corelib_http_init();
}

pub(crate) fn init() {
    unsafe { pxs_corelib_http_init() };
}