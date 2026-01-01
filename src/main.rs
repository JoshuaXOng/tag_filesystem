use std::process::exit;

use clap::Parser;
use tag_filesystem::{cli::ProgramParameters, errors::ResultBtAny};

fn main() -> ResultBtAny<()> {
    if let Err(e) = _main() {
        println!("{}", e.to_string_wbt());
        exit(1);
    }
    Ok(())
}

fn _main() -> ResultBtAny<()> {
    let mut program_arguments = ProgramParameters::parse();
    program_arguments.run()
}
