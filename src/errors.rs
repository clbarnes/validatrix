use std::fmt::{Display, Write};

use crate::diagnostics::Diagnostics;

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

const INDENT: &str = "   ";

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Validation failure(s):")?;
        for fa in self.0.iter() {
            f.write_str("\n")?;
            f.write_str(INDENT)?;
            fa.fmt(f)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

/// Validation error type wrapping a list of [Failure]s.
#[derive(Debug)]
pub struct Error(pub(crate) Diagnostics);

impl Error {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Struct representing a single validation failure.
/// Used to build informative error messages for [Error].
#[derive(Debug, Clone)]
pub struct Failure {
    pub(crate) key: Vec<Key>,
    // todo: replace with Cow?
    pub(crate) message: String,
}

impl Failure {
    pub fn new(path: &[Key], msg: impl Into<String>) -> Self {
        Self {
            key: path.to_vec(),
            message: msg.into(),
        }
    }
}

impl Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('$')?;
        for c in self.key.iter() {
            match c {
                Key::Index(n) => f.write_fmt(format_args!("[{n}]"))?,
                Key::Field(s) => {
                    f.write_char('.')?;
                    f.write_str(s)?;
                }
            }
        }
        f.write_str(": ")?;
        f.write_str(&self.message)
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum Key {
    Index(usize),
    // todo: also CoW?
    Field(&'static str),
}

impl From<usize> for Key {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

impl From<&'static str> for Key {
    fn from(value: &'static str) -> Self {
        Self::Field(value)
    }
}
