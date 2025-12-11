#![cfg_attr(test, allow(unused))]

use std::{error::Error, fmt::Display};

// TODO: What how is this possible.
use derive_more::{Display, Error};
use drums::{define_with_backtrace, Backtrace};
use tracing::{error, info};
use tracing_test::traced_test;

#[test]
#[traced_test]
fn converting_errors_to_error_a_with_backtrace() {
    let e = a_to_bt_a().unwrap_err();
    info!("{e:#?}");
    let e = b_to_bt_a().unwrap_err();
    info!("{e:#?}");
    let e = c_to_bt_a().unwrap_err();
    info!("{e:#?}");
    let e = external_to_bt_a().unwrap_err();
    info!("{e:#?}");

    let e = bt_a_to_bt_a().unwrap_err();
    info!("{e:#?}");
    let e = bt_a_to_bt_a().unwrap_err();
    info!("{e:#?}");
    let e = bt_c_to_bt_a().unwrap_err();
    info!("{e:#?}");
    let e = bt_external_to_bt_a().unwrap_err();
    info!("{e:#?}");
}

fn a_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = ErrorA;
    Err(e)?;
    Ok(())
}

fn b_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = ErrorB;
    Err(e)?;
    Ok(())
}

fn c_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = ErrorC::One("test".into());
    Err(e)?;
    Ok(())
}

fn external_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = std::io::Error::other("test");
    Err(e)?;
    Ok(())
}

fn bt_a_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = WithBacktrace::new(ErrorA);
    Err(e)?;
    Ok(())
}

fn bt_b_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = WithBacktrace::new(ErrorB);
    Err(e)?;
    Ok(())
}

fn bt_c_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = WithBacktrace::new(ErrorC::One("test".into()));
    Err(e)?;
    Ok(())
}

fn bt_external_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = WithBacktrace::new(std::io::Error::other("test"));
    Err(e)?;
    Ok(())
}

// TODO: Write assertions
#[test]
#[traced_test]
fn converting_errors_to_error_dyn_with_backtrace() {
    let e = a_to_bt_dyn().unwrap_err();
    info!("{e:#?}");
    let e = b_to_bt_dyn().unwrap_err();
    info!("{e:#?}");
    let e = c_to_bt_dyn().unwrap_err();
    info!("{e:#?}");
    let e = external_to_bt_dyn().unwrap_err();
    info!("{e:#?}");

    let e = bt_a_to_bt_dyn().unwrap_err();
    info!("{e:#?}");
    let e = bt_a_to_bt_dyn().unwrap_err();
    info!("{e:#?}");
    let e = bt_c_to_bt_dyn().unwrap_err();
    info!("{e:#?}");
    let e = bt_external_to_bt_dyn().unwrap_err();
    info!("{e:#?}");
}

fn a_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = ErrorA;
    Err(e)?;
    Ok(())
}

fn b_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = ErrorB;
    Err(e)?;
    Ok(())
}

fn c_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = ErrorC::One("test".into());
    Err(e)?;
    Ok(())
}

fn external_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = IoError::other("test");
    Err(e)?;
    Ok(())
}

fn bt_a_to_bt_dyn() -> Result<(), WithBacktrace<ErrorA>> {
    let e = WithBacktrace::new(ErrorA);
    Err(e)?;
    Ok(())
}

fn bt_b_to_bt_dyn() -> Result<(), WithBacktrace<ErrorA>> {
    let e = WithBacktrace::new(ErrorB);
    Err(e)?;
    Ok(())
}

fn bt_c_to_bt_dyn() -> Result<(), WithBacktrace<ErrorA>> {
    let e = WithBacktrace::new(ErrorC::One("test".into()));
    Err(e)?;
    Ok(())
}

fn bt_external_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = WithBacktrace::new(IoError::other("test"));
    Err(e)?;
    Ok(())
}

define_with_backtrace!();

// TODO: Should macro accept path?
use std::io::Error as IoError;

define_to_dyn!(IoError);

#[derive(Debug, Display, Error, Backtrace)]
#[display("ErrorA")]
#[bt_from(ErrorB, ErrorC, IoError)]
struct ErrorA;

impl From<ErrorB> for ErrorA {
    fn from(_: ErrorB) -> Self {
        ErrorA
    }
}

impl From<ErrorC> for ErrorA {
    fn from(_: ErrorC) -> Self {
        ErrorA
    }
}

impl From<IoError> for ErrorA {
    fn from(_: IoError) -> Self {
        ErrorA
    }
}

#[derive(Debug, Display, Error, Backtrace)]
#[display("ErrorB")]
struct ErrorB;

#[derive(Debug, Display, Error, Backtrace)]
#[display("ErrorC::{_variant}")]
enum ErrorC {
    One(Box<dyn Error>),
    Two
}
