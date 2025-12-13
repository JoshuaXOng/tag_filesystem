pub mod change;
pub mod setup;

use clap::{Parser, Subcommand};

use crate::{cli::{tags::{change::ChangeParameters, setup::SetupParameters}, ProgramParameters},
    errors::ResultBtAny};

#[derive(Parser, Debug)]
pub struct TagsParameters {
    #[command(subcommand)]
    pub subcommand: TagsSubcommand
}

impl TagsParameters {
    pub fn run(&self, program_arguments: &ProgramParameters) -> ResultBtAny<()> {
        match &self.subcommand {
            TagsSubcommand::Setup(setup_arguments) =>setup_arguments.run(program_arguments),
            TagsSubcommand::Change(change_arguments) => change_arguments.run()
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum TagsSubcommand {
    Change(ChangeParameters),
    Setup(SetupParameters)
}
