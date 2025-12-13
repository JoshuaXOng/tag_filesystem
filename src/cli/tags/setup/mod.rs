use std::{env::current_exe, fs::{create_dir_all, File}, io::Write, path::PathBuf};

use askama::Template;
use clap::Parser;
use tracing::info;

use crate::{cli::ProgramParameters, errors::{AnyError, ResultBtAny}, path_::get_configuration_directory};

#[derive(Parser, Debug)]
pub struct SetupParameters {
    #[arg(short, long, default_value = get_configuration_directory()
        .join(DEFAULT_SCRIPT_NAME)
        .into_os_string())]
    script_path: PathBuf,
    #[arg(short, long, default_value = "ct")]
    wrapper_name: String
}

pub const DEFAULT_SCRIPT_NAME : &str = "change_tags.sh";

impl SetupParameters {
    pub fn run(&self, program_arguments: &ProgramParameters) -> ResultBtAny<()> {
        let script_content = ChangeTemplate::try_from(self)?
            .render()?;

        if program_arguments.dry {
            println!("Would have written `{}` to `{}`.", script_content,
                self.script_path.to_string_lossy());
        } else {
            create_dir_all(&self.script_path.parent().expect("To have a parent."))?;
            info!("Creating all directories up until `{}`.", self.script_path.to_string_lossy());
            File::create(&self.script_path)?
                .write_all(script_content.as_bytes())?;
            info!("Wrote `{}` to `{}`.", script_content, self.script_path.to_string_lossy());
            println!("Remember to add `. {}` to `.bashrc` or an equivalent.",
                self.script_path.to_string_lossy());
        }
        Ok(())
    }
}

#[derive(Template)]
#[template(path = "change_tags.sh.j2")] 
pub struct ChangeTemplate {
    wrapper_name: String,
    to_binary: String
}

impl TryFrom<&SetupParameters> for ChangeTemplate {
    type Error = AnyError;

    fn try_from(value: &SetupParameters) -> Result<Self, Self::Error> {
        Ok(Self {
            wrapper_name: value.wrapper_name.clone(),
            to_binary: current_exe()?
                .to_string_lossy()
                .into_owned()
        })
    }
}
