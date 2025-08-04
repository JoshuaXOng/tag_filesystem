use std::error::Error;

use clap::Parser;
use rustyline::{history::MemHistory, Config, Editor};
use whoami::fallible::{hostname, username};

use crate::commands::{CurrentLabels, ProgramParameters};

pub struct Shell {
    _command_history: Vec<String>,
    current_labels: CurrentLabels,
    _command_input: Vec<String>,
    rustyline_engine: Editor<(), MemHistory>,
}

impl Shell {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            _command_history: vec![],
            current_labels: CurrentLabels::new(),
            _command_input: vec![],
            rustyline_engine: Editor::with_history(
                Config::builder().build(),
                MemHistory::new()
            )?,
        })
    }

    pub fn run_loop(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let user_input = self.rustyline_engine.readline(&self.get_prompt())?;
            self.rustyline_engine.add_history_entry(&user_input)?;
            self.handle_input(&user_input)?; 
        }
    }

    fn get_prompt(&self) -> String {
        let user_name = username()
            .unwrap_or(String::from("???"));
        let hostname = hostname()
            .unwrap_or(String::from("???"));
        format!(
            "{}@{}:{{ {} }}$ ", 
            user_name,
            hostname,
            self.current_labels
        )
    }

    fn handle_input(&mut self, user_input: &str) -> Result<(), Box<dyn Error>> {
        let user_input = user_input.split(" ");
        let x = ProgramParameters::try_parse_from(user_input);
        println!("{x:?}");
//        let user_input = match self.input_parser
//        .try_get_matches_from_mut(user_input) {
//            Ok(input) => input,
//            Err(_e) => {
//                println!("{}", self.input_parser.render_usage());
//                return Ok(());
//            },
//        };
//
//        match user_input.subcommand() {
//            Some((x, y)) => {
//                println!("{x}, {y:?}");
//            },
//            None => {
//                println!("{}", self.input_parser.render_usage());
//                return Ok(());
//            },
//        };

        Ok(())
    }
}
