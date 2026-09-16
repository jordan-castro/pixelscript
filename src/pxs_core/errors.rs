#[macro_export]
macro_rules! expected_argc {
    ($args:expr, $exp:expr) => {{
        let num = pxs_argc($args);
        if num < $exp {
            let mut cstring = CStringSafe::new();
            return pxs_newexception(cstring.new_string(&format!("Expected {}, found: {}", $exp, num)));
        }
    }};
}