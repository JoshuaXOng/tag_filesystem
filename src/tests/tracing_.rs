use std::{io::{self, Write}, sync::OnceLock};
use tracing::info;
use tracing_subscriber::fmt::MakeWriter;

use crate::tracing_::configure_tracing;

static FOR_TRACING_SETUP: OnceLock<()> = OnceLock::new();

pub fn setup_tracing() {
    FOR_TRACING_SETUP.get_or_init(|| {
        let tracing_setup = configure_tracing();
        tracing_setup.0.with_writer(TestWriter)
            .init();
        info!("Tracing directive is `{}`.", tracing_setup.1);
    });
}

pub struct TestWriter;

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match str::from_utf8(buf) {
            Ok(to_write) => {
                to_write.lines()
                    .for_each(|line| println!("{line}"));
                Ok(buf.len())
            },
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData, e))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for TestWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        TestWriter
    }
}
