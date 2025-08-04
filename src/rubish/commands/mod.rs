use std::{collections::HashSet, fmt::Display};

use clap::{arg, command, Args, Parser, Subcommand};

pub struct CurrentLabels(HashSet<String>);

impl CurrentLabels {
    pub fn new() -> Self {
        Self(HashSet::new())
    }
}

impl Display for CurrentLabels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|label| label.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Parser, Debug)]
pub struct ProgramParameters {
    #[command(subcommand)]
    pub workers: Workers
}

#[derive(Subcommand, Debug)]
pub enum Workers {
    Ls(LsParamereters),
    Cd(CdParameters)
}

#[derive(Args, Debug)]
pub struct LsParamereters {
    #[arg(short = 'a')]
    pub add: String
}

#[derive(Args, Debug)]
pub struct CdParameters {
    #[arg(short = 'a')]
    pub add: String
}
