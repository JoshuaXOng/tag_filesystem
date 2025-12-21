use std::{env::{self, current_exe}, ffi::CString, io::{stderr, Stderr}, path::{Path, PathBuf}, sync::Mutex};

use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
use syslog_tracing::Syslog;
use tracing::info;
use tracing_subscriber::{fmt::{format::{DefaultFields, Format}, SubscriberBuilder}, EnvFilter};

use crate::{errors::ResultBtAny, path::get_configuration_directory};

const LOG_CONFIG_ENVVAR: &str = "ADDITIONAL_LOG_DIRECTIVES";

const SETUP_TRACING_EXPECTATION: &str = "That it's ok to crash if can't setup tracing.";

pub fn configure_tracing()
-> (SubscriberBuilder<DefaultFields, Format, EnvFilter, fn() -> Stderr>, String) {
    let to_binary = current_exe()
        .expect(SETUP_TRACING_EXPECTATION);
    let binary_name = to_binary
        .file_name()
        .expect(SETUP_TRACING_EXPECTATION)
        .to_string_lossy();
    let mut directive = format!("warn,{}=info,{}=info", env!("CARGO_PKG_NAME"),
        binary_name);
    let overrides = env::var(LOG_CONFIG_ENVVAR);
    if let Ok(overrides) = overrides && !overrides.is_empty() {
        directive += &format!(",{}", overrides);
    }

    (
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::builder()
                .parse(&directive)
                .expect(SETUP_TRACING_EXPECTATION))
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .with_writer(stderr),
        directive
    )
}

pub fn setup_normal_tracing<P: AsRef<Path>>(middle_scoping: P) {
    let tracing_setup = configure_tracing();
    let rotating_writer = Mutex::new(FileRotate::new(
        get_configuration_directory()
            .join("logs")
            .join(middle_scoping)
            .join("tfs.log"),
        AppendCount::new(1),
        ContentLimit::Bytes(10_000_000),
        Compression::None,
        None
    ));
    tracing_setup.0
        .with_writer(rotating_writer)
        .init();
    info!("Tracing directive is `{}`.", tracing_setup.1);
}

pub fn setup_syslog_tracing() -> ResultBtAny<()> {
    let tracing_setup = configure_tracing();
    let (syslog_options, syslog_facility) = Default::default();
    let logger_name = env!("CARGO_PKG_NAME");
    let syslog_writer = Syslog::new(
        CString::new(logger_name)?,
        syslog_options, syslog_facility)
        .ok_or(format!("Syslog for `{logger_name}` already exists."))?;
    tracing_setup.0
        .with_writer(syslog_writer)
        .init();
    info!("Tracing directive is `{}`.", tracing_setup.1);
    Ok(())
}

#[test]
fn checking_join_behaviour() {
    let mut path = PathBuf::from("/home")
        .join("")
        .join("jxo");
    assert_eq!(path, PathBuf::from("/home").join("jxo"));
}
