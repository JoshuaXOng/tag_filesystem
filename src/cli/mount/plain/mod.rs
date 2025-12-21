use std::fs::create_dir_all;

use clap::Args;
use tracing::info;

use crate::{cli::{mount::MountParameters, ProgramParameters}, errors::ResultBtAny,
    filesystem::TagFilesystem};

#[derive(Args, Debug)]
pub struct PlainParameters;

impl PlainParameters {
    pub fn run(&self, program_arguments: &ProgramParameters,
        mount_arguments: &MountParameters) -> ResultBtAny<()>
    {
        let _mount_path = mount_arguments.mount_path.to_string_lossy();
        if program_arguments.dry {
            info!("Would have created directories on the way to, and mounted TFS \
                at `{}`.", _mount_path);
        } else {
            create_dir_all(&mount_arguments.mount_path)?;
            info!("Creating all directories to `{}`.", _mount_path);
            TagFilesystem::run_filesystem(&mount_arguments.mount_path)?;
        }
        Ok(())
    }
}

