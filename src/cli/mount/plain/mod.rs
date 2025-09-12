use std::{fs::create_dir_all, path::PathBuf};

use clap::Args;
use fuser::{mount2, MountOption};
use tracing::info;

use crate::{cli::ProgramParameters, errors::Result_, filesystem::TagFilesystem};

#[derive(Args, Debug)]
pub struct PlainParameters {
    pub mount_path: PathBuf
}

impl PlainParameters {
    pub fn run(&self, program_arguments: &ProgramParameters) -> Result_<()> {
        let _mount_path = self.mount_path.to_string_lossy();
        if program_arguments.dry {
            info!("Would have created directories on the way to, and mounted TFS \
                at `{}`.", _mount_path);
        } else {
            create_dir_all(&self.mount_path)?;
            info!("Creating all directories to `{}`.", _mount_path);
            mount2(TagFilesystem::try_new(&self.mount_path)?,
                &self.mount_path,
                &[MountOption::AutoUnmount, MountOption::AllowRoot])?;
            info!("Mounted TFS at `{}`.", _mount_path);
        }
        Ok(())
    }
}

