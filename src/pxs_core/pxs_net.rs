unsafe extern "C" {
    fn pxs_corelib_net_init();
}

pub(crate) fn init() {
    unsafe { pxs_corelib_net_init() };
}