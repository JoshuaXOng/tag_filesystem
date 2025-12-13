use std::{fs::create_dir_all, path::PathBuf};

use clap::Args;
use tracing::info;

use crate::{cli::ProgramParameters, errors::ResultBtAny, filesystem::TagFilesystem};

#[derive(Args, Debug)]
pub struct PlainParameters {
    pub mount_path: PathBuf
}

impl PlainParameters {
    pub fn run(&self, program_arguments: &ProgramParameters) -> ResultBtAny<()> {
        let _mount_path = self.mount_path.to_string_lossy();
        if program_arguments.dry {
            info!("Would have created directories on the way to, and mounted TFS \
                at `{}`.", _mount_path);
        } else {
            create_dir_all(&self.mount_path)?;
            info!("Creating all directories to `{}`.", _mount_path);
            TagFilesystem::run_filesystem(&self.mount_path)?;
        }
        Ok(())
    }
}

