unsafe extern "C" {
    fn pxs_corelib_http_init();
}

pub(crate) fn init() {
    unsafe { pxs_corelib_http_init() };
}