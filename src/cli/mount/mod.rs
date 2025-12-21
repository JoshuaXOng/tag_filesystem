pub mod plain;
pub mod systemd;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{cli::{mount::{plain::PlainParameters, systemd::SystemdParamereters},
    ProgramParameters}, errors::ResultBtAny, tracing::setup_normal_tracing, wrappers::PathExt};

#[derive(Parser, Debug)]
pub struct MountParameters {
    pub mount_path: PathBuf,
    #[command(subcommand)]
    pub subcommand: MountSubcommand
}

impl MountParameters {
    pub fn run(&self, program_arguments: &ProgramParameters) -> ResultBtAny<()> {
        setup_normal_tracing(self.mount_path.__strip_prefix("/"));

        match &self.subcommand {
            MountSubcommand::Systemd(systemd_argument) =>
                systemd_argument.run(program_arguments, &self),
            MountSubcommand::Plain(plain_arguments) =>
                plain_arguments.run(program_arguments, &self)
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum MountSubcommand {
    Plain(PlainParameters),
    Systemd(SystemdParamereters)
}
