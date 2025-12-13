#![cfg_attr(test, allow(unused))]

use std::{env, error::Error, fmt::Display, io::Error as IoError, sync::OnceLock};

// TODO: What, how is this possible?
use derive_more::{Display, Error};
use drums::{define_with_backtrace, Backtrace};
use tracing::{error, info};
use tracing_test::traced_test;

static FOR_BACKTRACE_ENABLE: OnceLock<()> = OnceLock::new();

fn enable_backtrace() {
    FOR_BACKTRACE_ENABLE.get_or_init(|| {
        unsafe {
            env::set_var("RUST_BACKTRACE", "1");
        }
    });
}

const MODULE_NAME: &str = "backtrace";

macro_rules! assert_backtrace {
    ($error_message: expr, $original_function: expr, $original_line: expr,
    $invocation_function: expr, $invocation_line: expr) => {
        assert!(logs_contain(&format!("{}: {}: WithBacktrace {{",
            $invocation_function, MODULE_NAME)));
        assert!(logs_contain(&format!("error: {},", $error_message)));
        assert!(logs_contain("backtrace: Backtrace ["));
        assert!(logs_contain(&format!(
            r#"{{ fn: "{}::{}", file: "./tests/{}.rs", line: {} }},"#,
            MODULE_NAME, $original_function, MODULE_NAME, $original_line)));
        assert!(logs_contain(&format!(
            r#"{{ fn: "{}::{}", file: "./tests/{}.rs", line: {} }},"#,
            MODULE_NAME, $invocation_function, MODULE_NAME, $invocation_line)));
    }
}

#[test]
#[traced_test]
fn running_a_to_bt_a() {
    enable_backtrace();
    let e = a_to_bt_a().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(A2BTA_DEBUG, "a_to_bt_a", A2BTA_LINE, 
        "running_a_to_bt_a", e_line);
}
const A2BTA_DEBUG: &str = "ErrorA";
const A2BTA_LINE: u32 = line!() + 3;
fn a_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = ErrorA;
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_b_to_bt_a() {
    enable_backtrace();
    let e = b_to_bt_a().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(B2BTA_DEBUG, "b_to_bt_a", B2BTA_LINE, 
        "running_b_to_bt_a", e_line);
}
const B2BTA_DEBUG: &str = "ErrorA";
const B2BTA_LINE: u32 = line!() + 3;
fn b_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = ErrorB;
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_c_to_bt_a() {
    enable_backtrace();
    let e = c_to_bt_a().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(C2BTA_DEBUG, "c_to_bt_a", C2BTA_LINE, 
        "running_c_to_bt_a", e_line);
}
const C2BTA_DEBUG: &str = "ErrorA";
const C2BTA_LINE: u32 = line!() + 3;
fn c_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = ErrorC::One("test".into());
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_external_to_bt_a() {
    enable_backtrace();
    let e = external_to_bt_a().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(EXT2BTA_DEBUG, "external_to_bt_a", EXT2BTA_LINE, 
        "running_external_to_bt_a", e_line);
}
const EXT2BTA_DEBUG: &str = "ErrorA";
const EXT2BTA_LINE: u32 = line!() + 3;
fn external_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = std::io::Error::other("test");
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_bt_a_to_bt_a() {
    enable_backtrace();
    let e = bt_a_to_bt_a().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(BTA2BTA_DEBUG, "bt_a_to_bt_a", BTA2BTA_LINE, 
        "running_bt_a_to_bt_a", e_line);
}
const BTA2BTA_DEBUG: &str = "ErrorA";
const BTA2BTA_LINE: u32 = line!() + 2;
fn bt_a_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = WithBacktrace::new(ErrorA);
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_bt_b_to_bt_a() {
    enable_backtrace();
    let e = bt_b_to_bt_a().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(BTB2BTA_DEBUG, "bt_b_to_bt_a", BTB2BTA_LINE, 
        "running_bt_b_to_bt_a", e_line);
}
const BTB2BTA_DEBUG: &str = "ErrorA";
const BTB2BTA_LINE: u32 = line!() + 2;
fn bt_b_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = WithBacktrace::new(ErrorB);
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_bt_c_to_bt_a() {
    enable_backtrace();
    let e = bt_c_to_bt_a().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(BTC2BTA_DEBUG, "bt_c_to_bt_a", BTC2BTA_LINE, 
        "running_bt_c_to_bt_a", e_line);
}
const BTC2BTA_DEBUG: &str = "ErrorA";
const BTC2BTA_LINE: u32 = line!() + 2;
fn bt_c_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = WithBacktrace::new(ErrorC::One("test".into()));
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_bt_external_to_bt_a() {
    enable_backtrace();
    let e = bt_external_to_bt_a().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(BTEXT2BTA_DEBUG, "bt_external_to_bt_a", BTEXT2BTA_LINE, 
        "running_bt_external_to_bt_a", e_line);
}
const BTEXT2BTA_DEBUG: &str = "ErrorA";
const BTEXT2BTA_LINE: u32 = line!() + 2;
fn bt_external_to_bt_a() -> Result<(), WithBacktrace<ErrorA>> {
    let e = WithBacktrace::new(std::io::Error::other("test"));
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_a_to_bt_dyn() {
    enable_backtrace();
    let e = a_to_bt_dyn().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(A2BTDYN_DEBUG, "a_to_bt_dyn", A2BTDYN_LINE,
        "running_a_to_bt_dyn", e_line);
}
const A2BTDYN_DEBUG: &str = "ErrorA";
const A2BTDYN_LINE: u32 = line!() + 3;
fn a_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = ErrorA;
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_b_to_bt_dyn() {
    enable_backtrace();
    let e = b_to_bt_dyn().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(B2BTDYN_DEBUG, "b_to_bt_dyn", B2BTDYN_LINE,
        "running_b_to_bt_dyn", e_line);
}
const B2BTDYN_DEBUG: &str = "ErrorB";
const B2BTDYN_LINE: u32 = line!() + 3;
fn b_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = ErrorB;
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_c_to_bt_dyn() {
    enable_backtrace();
    let e = c_to_bt_dyn().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(C2BTDYN_DEBUG, "c_to_bt_dyn", C2BTDYN_LINE,
        "running_c_to_bt_dyn", e_line);
}
const C2BTDYN_DEBUG: &str = r#"One("test")"#;
const C2BTDYN_LINE: u32 = line!() + 3;
fn c_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = ErrorC::One("test".into());
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_external_to_bt_dyn() {
    enable_backtrace();
    let e = external_to_bt_dyn().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(EXT2BTDYN_DEBUG,  "external_to_bt_dyn", EXT2BTDYN_LINE,
        "running_external_to_bt_dyn", e_line);
}
const EXT2BTDYN_DEBUG: &str = r#"Custom { kind: Other, error: "test" }"#;
const EXT2BTDYN_LINE: u32 = line!() + 3;
fn external_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = IoError::other("test");
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_bt_a_to_bt_dyn() {
    enable_backtrace();
    let e = bt_a_to_bt_dyn().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(BTA2BTDYN_DEBUG, "bt_a_to_bt_dyn", BTA2BTDYN_LINE,
        "running_bt_a_to_bt_dyn", e_line);
}
const BTA2BTDYN_DEBUG: &str = "ErrorA";
const BTA2BTDYN_LINE: u32 = line!() + 2;
fn bt_a_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = WithBacktrace::new(ErrorA);
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_bt_b_to_bt_dyn() {
    enable_backtrace();
    let e = bt_b_to_bt_dyn().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(BTB2BTDYN_DEBUG, "bt_b_to_bt_dyn", BTB2BTDYN_LINE,
        "running_bt_b_to_bt_dyn", e_line);
}
const BTB2BTDYN_DEBUG: &str = "ErrorB";
const BTB2BTDYN_LINE: u32 = line!() + 2;
fn bt_b_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = WithBacktrace::new(ErrorB);
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_bt_c_to_bt_dyn() {
    enable_backtrace();
    let e = bt_c_to_bt_dyn().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(BTC2BTDYN_DEBUG, "bt_c_to_bt_dyn", BTC2BTDYN_LINE, 
        "running_bt_c_to_bt_dyn", e_line);
}
const BTC2BTDYN_DEBUG: &str = r#"One("test")"#;
const BTC2BTDYN_LINE: u32 = line!() + 2;
fn bt_c_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = WithBacktrace::new(ErrorC::One("test".into()));
    Err(e)?;
    Ok(())
}

#[test]
#[traced_test]
fn running_bt_external_to_bt_dyn() {
    enable_backtrace();
    let e = bt_external_to_bt_dyn().unwrap_err(); let e_line = line!();
    info!("{e:?}");
    assert_backtrace!(BTEXT2BTDYN_DEBUG, "bt_external_to_bt_dyn", BTEXT2BTDYN_LINE,
        "running_bt_external_to_bt_dyn", e_line);
}
const BTEXT2BTDYN_DEBUG: &str = r#"Custom { kind: Other, error: "test" }"#;
const BTEXT2BTDYN_LINE: u32 = line!() + 2;
fn bt_external_to_bt_dyn() -> Result<(), WithBacktrace<Box<dyn Error>>> {
    let e = WithBacktrace::new(IoError::other("test"));
    Err(e)?;
    Ok(())
}

define_with_backtrace!();

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
    #[display("One({_0})")]
    One(Box<dyn Error>),
    Two
}
