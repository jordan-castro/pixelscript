#[cfg(feature = "pxs-debug")]
#[macro_export]
macro_rules! pxs_debug {
    ($($arg:tt)*) => { eprintln!("[PXS_DEBUG] {}", format_args!($($arg)*)); }
}

#[cfg(not(feature = "pxs-debug"))]
#[macro_export]
macro_rules! pxs_debug {
    ($($arg:tt)*) => {};
}