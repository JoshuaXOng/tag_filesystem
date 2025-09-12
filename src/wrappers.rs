use std::{collections::{BTreeSet, HashSet}, fmt::{self, Display, Formatter}, path::Path};

pub struct VecWrapper<T>(pub Vec<T>);

impl<T: Display> Display for VecWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_iter(f, ('[', ']'), self.0.iter())
    }
}

pub struct HashSetWrapper<T>(pub HashSet<T>);

impl<T: Display> Display for HashSetWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_iter(f, ('{', '}'), self.0.iter())
    }
}

pub fn write_iter<T: Display>
(f: &mut Formatter,
    terminals: (char, char),
    to_write: impl Iterator<Item = T>)
-> fmt::Result {
    let mut to_write = to_write.peekable();
    let was_not_empty = to_write.peek().is_some();

    let is_terminals_curly = terminals.0 == '{' && terminals.1 == '}';
    let should_add_padding = was_not_empty && is_terminals_curly;

    write!(f, "{}", terminals.0)?;
    for (index, to_display) in to_write.enumerate() {
        let is_not_first = index != 0;
        if is_not_first { write!(f, ", ")? }
        else if should_add_padding { write!(f, " ")? };

        write!(f, "{}", to_display)?;
    };
    if should_add_padding {
        write!(f, " ")?;
    }
    write!(f, "{}", terminals.1)
}

pub fn write_btreeset<T: Display> (f: &mut Formatter, to_write: &BTreeSet<T>) -> fmt::Result {
    write_iter(f, ('{', '}'), to_write.iter())
}

pub trait PathExt {
    fn __strip_prefix<P>(&self, base: P) -> &Path where P: AsRef<Path>;
}

impl PathExt for Path {
    fn __strip_prefix<P>(&self, base: P) -> &Path where P: AsRef<Path> {
        self.strip_prefix(base).unwrap_or(self)
    }
}

pub trait StrExt {
    fn __strip_prefix(&self, prefix: &str) -> &str;
}

impl StrExt for str {
    fn __strip_prefix(&self, prefix: &str) -> &str {
        self.strip_prefix(prefix).unwrap_or(self)
    }
}
