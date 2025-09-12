use std::{env::{self, current_exe}, io::{stderr, Stderr}, sync::Mutex};

use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
use tracing::info;
use tracing_subscriber::{fmt::{format::{DefaultFields, Format}, SubscriberBuilder}, EnvFilter};

use crate::path_::get_configuration_directory;

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

pub fn setup_tracing() {
    let tracing_setup = configure_tracing();
    // TODO: Safe across processes?
    let rotating_writer = Mutex::new(FileRotate::new(
        get_configuration_directory().join("tfs.log"),
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
