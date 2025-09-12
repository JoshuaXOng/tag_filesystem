use clap::Parser;
use tag_filesystem::{cli::ProgramParameters, errors::Result_, tracing_::setup_tracing};

// TODO: More user friendly error messages.
// Error: Os { code: 21, kind: IsADirectory, message: "Is a directory" }
fn main() -> Result_<()> {
    setup_tracing();
    let program_arguments = ProgramParameters::parse();
    program_arguments.run()
}
