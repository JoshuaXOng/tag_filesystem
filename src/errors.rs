use std::error::Error;

use crate::WithBacktrace;

// TODO: Consistent file, lineno.
pub type ResultBt<T, E> = Result<T, WithBacktrace<E>>;
pub type ResultBtAny<T> = Result<T, WithBacktrace<AnyError>>;

define_to_dyn!(&str);
define_to_dyn!(String);
define_to_dyn!(std::num::TryFromIntError);
define_to_dyn!(std::io::Error);
define_to_dyn!(capnp::Error);
define_to_dyn!(serde_json::Error);
define_to_dyn!(askama::Error);

pub trait StringExt {
    fn append_if_error<T>(&mut self, r: ResultBtAny<T>);
}

impl StringExt for String {
    fn append_if_error<T>(&mut self, r: ResultBtAny<T>) {
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
