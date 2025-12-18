use std::process::exit;

use clap::Parser;
use tag_filesystem::{cli::ProgramParameters, errors::ResultBtAny, tracing::setup_tracing};

// TODO: More user friendly error messages.
// Error: Os { code: 21, kind: IsADirectory, message: "Is a directory" }
fn main() -> ResultBtAny<()> {
    if let Err(e) = _main() {
        println!("{}", e.to_string_wbt());
        exit(1);
    }
    Ok(())
}

fn _main() -> ResultBtAny<()> {
    setup_tracing();
    let program_arguments = ProgramParameters::parse();
    program_arguments.run()
}
