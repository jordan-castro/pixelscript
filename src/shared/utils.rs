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

#[macro_export]
/// Macro to wrap features
macro_rules! with_feature {
    ($feature:expr, $logic:block) => {
        #[cfg(feature=$feature)]
        {
            $logic
        }
    };
    ($feature:literal, $logic:block, $fallback:block) => {{
        #[cfg(feature = $feature)]
        {
            $logic
        }
        #[cfg(not(feature = $feature))]
        {
            $fallback
        }
    }};
}

