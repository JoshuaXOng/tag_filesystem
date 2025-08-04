use std::error::Error;

// TODO: See if can replace with either that one crate or
// own error chain w/ generics.
pub type Result_<T> = Result<T, AnyError>;

pub type AnyError = Box<dyn Error>;

#[macro_export]
macro_rules! unwrap_or {
    ($to_unwrap: expr, $e: ident, $else_do: expr) => {
        {
            match $to_unwrap {
                Ok(x) => x,
                Err($e) => _ = $else_do
            }
        }
    };
    ($to_unwrap: expr, $else_do: expr) => {
        {
            match $to_unwrap {
                Some(x) => x,
                None => _ = $else_do
            }
        }
    };
}
