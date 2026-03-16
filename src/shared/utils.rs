/// A useful macro for debuggin in pixelscript.
#[macro_export]
macro_rules! pxs_debug {
    ($($arg:tt)*) =>
    { 
        #[cfg(feature = "pxs-debug")] 
        {
            let loc = std::panic::Location::caller();
            eprintln!(
                "[PXS_DEBUG {}:{}] {}", 
                loc.file(),
                loc.line(),
                format_args!($($arg)*)
            ); 
        }
    }
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

