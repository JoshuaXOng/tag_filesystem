use std::{collections::HashSet, path::PathBuf};

use crate::errors::ResultBtAny;

pub trait PathBufExt {
    fn add_tags(&mut self, to_add: &str) -> ResultBtAny<()>;
    fn subtract_tags(&mut self, to_subtract: &str) -> ResultBtAny<()>;
}

impl PathBufExt for PathBuf {
    fn add_tags(&mut self, to_add: &str) -> ResultBtAny<()> {
        let mut current_tags: HashSet<_> = get_current_tags(self)?
            .collect();

        current_tags.extend(parse_tags(to_add));

        self.set_file_name(
            format_tags(current_tags.into_iter()));
        Ok(())
    }

    fn subtract_tags(&mut self, to_subtract: &str) -> ResultBtAny<()> {
        let mut current_tags: HashSet<_> = get_current_tags(self)?
            .collect();

        for tag_name in parse_tags(&to_subtract) {
            current_tags.remove(tag_name);
        }

        self.set_file_name(
            format_tags(current_tags.into_iter()));
        Ok(())
    }
}

pub fn get_current_tags(path: &PathBuf) -> ResultBtAny<impl Iterator<Item = &str>> {
    let current_tags = path.file_name()
        .or_else(|| path.parent()
            .and_then(|parent| parent.file_name()))
        .ok_or(format!(
            "Leaf and parent parts do not exits, `{path:?}`."))?;


    let current_tags = current_tags.to_str()
        .ok_or(format!(
            "Can't convert to `str`, `{path:?}`."))?;
    Ok(parse_tags(current_tags))
}

pub fn parse_tags(tag_string: &str) -> impl Iterator<Item = &str> {
    tag_string
        .trim_matches(|character| match character {
            '{' | '}' | ' ' => true,
            _ => false
        })
        .split(',')
        .map(str::trim)
        .filter(|tag| *tag != "")
}

pub fn format_tags<'a> (tag_tokens: impl Iterator<Item = &'a str>) -> String {
    let mut formated_tags = tag_tokens.collect::<Vec<_>>();
    formated_tags.retain(|tag| *tag != "");
    formated_tags.sort();
    formated_tags.dedup();
    let is_not_empty = formated_tags.len() > 0;

    let mut _formated_tags = formated_tags.join(", ");
    _formated_tags.insert_str(0, "{");
    if is_not_empty {
        _formated_tags.insert_str(1, " ");
        _formated_tags += " ";
    }
    _formated_tags += "}";
    _formated_tags
}

pub fn get_configuration_directory() -> PathBuf {
    PathBuf::from(shellexpand::tilde("~/.tag_filesystem").as_ref())
}
