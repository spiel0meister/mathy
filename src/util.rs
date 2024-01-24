#[macro_export]
macro_rules! error {
    ($error_kind:ident, $msg:literal $(, $value:expr)*) => {{
        Error::new(ErrorKind::$error_kind, format!($msg, $($value),*))
    }};
}

pub use error;
