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

#[macro_export]
macro_rules! coalesce {
    (helper $message: expr, $error: ident) => {
        if let Err(e) = $error { format!("{} {}", $message, e.to_string()) }
        else { $message.to_string() }
    };
    (helper $message: expr, $error: ident, $($errors: ident), +) => {
        if let Err(e) = $error { 
            let running = coalesce!(helper $message, $($errors), +);
            format!("{} {}", running, e.to_string())
        } else {
            coalesce!(helper $message, $($errors), +)
        }
    };
    ($message: expr, $($errors: ident), +) => {
        match ($($errors), +) {
            ($(Ok($errors)), +) => Ok(($($errors), +)),
            ($($errors), +) => Err(coalesce!(helper $message, $($errors), +))
        }
    }
}

// TODO/WIP: Choose
#[macro_export]
macro_rules! coalescerr {
    (helper $message: expr, $error: ident) => {
        if let Err(e) = $error { format!("{} {}", $message, e.to_string()) }
        else { $message.to_string() }
    };
    (helper $message: expr, $error: ident, $($errors: ident), +) => {
        if let Err(e) = $error { 
            let running = coalescerr!(helper $message, $($errors), +);
            format!("{} {}", running, e.to_string())
        } else {
            coalescerr!(helper $message, $($errors), +)
        }
    };
    ($message: expr, $($errors: ident), +) => {
        let ($($errors), +) = match ($($errors), +) {
            ($(Ok($errors)), +) => (($($errors), +)),
            ($($errors), +) => Err(coalescerr!(helper $message, $($errors), +))?
        };
    }
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

// TODO: create f! macro
