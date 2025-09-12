use std::{env::current_dir, path::PathBuf, str::FromStr};

use bon::Builder;
use clap::Parser;
use tracing::info;

use crate::{errors::Result_, path_::{format_tags, get_current_tags}, wrappers::StrExt};

#[derive(Parser, PartialEq, Debug)]
pub struct ChangeParameters {
    #[arg(short = 'n', default_value_t = false)]
    pub are_negated: bool,
    pub tags: Vec<ChangeTag>
}

impl ChangeParameters {
    pub fn run(&self) -> Result_<()> {
        println!("{}", get_changed_path(&self)?.to_string_lossy());
        Ok(())
    }
}

#[derive(Builder, PartialEq, Debug, Clone)]
#[builder(on(String, into))]
pub struct ChangeTag {
    #[builder(default = false)]
    pub is_negated: bool,
    pub name: String 
}

impl ChangeTag {
    const NEGATION_PREFIX: &str = "~";
}

impl FromStr for ChangeTag {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let is_negated = s.starts_with(Self::NEGATION_PREFIX);
        let tag_name = s.__strip_prefix(Self::NEGATION_PREFIX);
        Ok(Self {
            is_negated,
            name: tag_name.to_string(),
        })
    }
}

pub fn get_changed_path(change_arguments: &ChangeParameters) -> Result_<PathBuf> {
    let mut current_path = current_dir()?;
    info!("Current path is `{}`.", current_path.to_string_lossy());

    let directory_name = current_path.file_name()
        .ok_or(format!("Path `{}` has no file name.", current_path.to_string_lossy()))?
        .to_string_lossy();
    if !directory_name.starts_with('{') && !directory_name.ends_with('}') {
        current_path.push("{}")
    }
    
    let mut current_tags = get_current_tags(&current_path)?
        .collect::<Vec<_>>();
    for change_tag in &change_arguments.tags {
        if change_tag.is_negated ^ change_arguments.are_negated {
            current_tags.retain(|tag| tag != &change_tag.name);
        } else {
            current_tags.push(&change_tag.name);
        }
    }
    let current_tags = format_tags(current_tags.into_iter());

    current_path.pop();
    current_path.push(current_tags);
    Ok(current_path)
}
