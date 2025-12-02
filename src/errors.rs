use std::{backtrace::Backtrace, error::Error, ops::Deref};

// TODO: Consider replacing with either that one crate or
// own error chain w/ generics.
// TODO: Consistent file, lineno.
pub type Result_<T> = Result<T, AnyError>;

pub trait StringExt {
    fn append_if_error<T>(&mut self, r: Result_<T>);
}

impl StringExt for String {
    fn append_if_error<T>(&mut self, r: Result_<T>) {
        if let Err(e) = r {
            self.push_str(" ");
            self.push_str(&e.to_string());
        }
    }
}

// TODO: Read up on reasoning behind things that are not Send and Sync. And why does
// `into`/conversions not work for some cases when not including Sync
pub type AnyError = Box<dyn Error + Send + Sync>;

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
