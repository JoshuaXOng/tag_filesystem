pub mod mount;
pub mod tags;

use std::fs::{self, create_dir_all};

use clap::{Parser, Subcommand};
use crate::{cli::{mount::MountParameters, tags::TagsParameters}, errors::ResultBtAny,
    path::get_configuration_directory};

#[derive(Parser, Debug)]
pub struct ProgramParameters {
    #[arg(short, long)]
    pub dry: bool,
    #[command(subcommand)]
    pub subcommand: ProgramSubcommands 
}

impl ProgramParameters {
    pub fn run(&mut self) -> ResultBtAny<()> {
        let configuration_directory = get_configuration_directory();
        create_dir_all(configuration_directory)?;

        if let ProgramSubcommands::Mount(ref mut mount_arguments) = self.subcommand {
            mount_arguments.mount_path = fs::canonicalize(&mount_arguments.mount_path)?;
        }

        match &self.subcommand {
            ProgramSubcommands::Mount(mount_arguments) => mount_arguments.run(self),
            ProgramSubcommands::Tags(tag_arguments) => tag_arguments.run(self)
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum ProgramSubcommands {
    Mount(MountParameters),
    Tags(TagsParameters)
}
