#[macro_export]
macro_rules! hypr_log {
    ($($arg:tt)*) => {
        $crate::helpers::Logger::log(&format!($($arg)*))
    };
}
