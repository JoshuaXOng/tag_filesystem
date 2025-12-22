use std::{error::Error, fmt::Display};

use crate::WithBacktrace;

pub type ResultBt<T, E> = Result<T, WithBacktrace<E>>;
pub type ResultBtAny<T> = Result<T, WithBacktrace<AnyError>>;

define_to_dyn!(&str);
define_to_dyn!(String);
define_to_dyn!(std::num::TryFromIntError);
define_to_dyn!(std::io::Error);
define_to_dyn!(std::ffi::NulError);
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

pub fn collect_errors<T, E: Display>(errors: impl Iterator<Item = ResultBt<T, E>>)
    -> ResultBtAny<()>
{
    let errors = errors.filter_map(Result::err)
        .map(|e| e.to_string())
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        Err(errors.join(" "))?
    }
    Ok(())
}

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
