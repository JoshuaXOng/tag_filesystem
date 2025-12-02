pub mod plain;
pub mod systemd;

use clap::{Parser, Subcommand};

use crate::{cli::{mount::{plain::PlainParameters, systemd::SystemdParamereters},
    ProgramParameters}, errors::Result_};

#[derive(Parser, Debug)]
pub struct MountParameters {
    #[command(subcommand)]
    pub subcommand: MountSubcommand
}

impl MountParameters {
    pub fn run(&self, program_arguments: &ProgramParameters) -> Result_<()> {
        match &self.subcommand {
            MountSubcommand::Systemd(systemd_argument) =>
                systemd_argument.run(program_arguments),
            MountSubcommand::Plain(plain_arguments) =>
                plain_arguments.run(program_arguments)
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum MountSubcommand {
    Plain(PlainParameters),
    Systemd(SystemdParamereters)
}
