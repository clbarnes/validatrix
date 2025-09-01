use std::fmt::{Display, Write};

use crate::Validate;

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
pub struct Error(Vec<Failure>);

impl Error {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<Accumulator> for Result<(), Error> {
    fn from(value: Accumulator) -> Self {
        if value.failures.is_empty() {
            Ok(())
        } else {
            Err(Error(value.failures))
        }
    }
}

#[derive(Debug, Default)]
pub struct Accumulator {
    pub prefix: Vec<Key>,
    failures: Vec<Failure>,
}

impl Accumulator {
    /// Add one extra failure to this accumulator, under the given keys.
    pub fn add_failure(&mut self, mut failure: Failure, keys: &[Key]) {
        for k in keys.iter() {
            failure.key.push(*k);
        }
        for k in self.prefix.iter().rev() {
            failure.key.push(*k);
        }
        self.failures.push(failure);
    }

    /// Ingest a whole error response into this accumulator, under the given keys.
    /// 
    /// If a failure was added, returns `true`.
    pub fn accumulate_err(&mut self, res: Result<(), Error>, keys: &[Key]) -> bool {
        let Err(e) = res else {
            return false;
        };
        for f in e.0 {
            self.add_failure(f, keys);
        }
        true
    }

    /// If a failure was added, returns > 0
    pub fn validate_iter<'a, V: Validate + 'a, I: IntoIterator<Item = &'a V>, K: Into<Key>>(
        &mut self,
        key: K,
        items: I,
    ) -> usize {
        let orig = self.len();
        self.prefix.push(key.into());
        for (idx, item) in items.into_iter().enumerate() {
            self.prefix.push(idx.into());
            item.validate_inner(self);
            self.prefix.pop();
        }
        self.prefix.pop();
        self.len() - orig
    }

    pub fn len(&self) -> usize {
        self.failures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.failures.is_empty()
    }
}

/// Struct representing a single validation failure.
/// Used to build informative error messages for [Error].
#[derive(Debug)]
pub struct Failure {
    key: Vec<Key>,
    // todo: replace with Cow?
    message: String,
}

impl Failure {
    pub fn with_key(mut self, key: Key) -> Self {
        self.key.push(key);
        self
    }
}

impl<T: Into<String>> From<T> for Failure {
    fn from(value: T) -> Self {
        Self {
            key: Default::default(),
            message: value.into(),
        }
    }
}

impl Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('$')?;
        for c in self.key.iter().rev() {
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

impl From<Failure> for Error {
    fn from(value: Failure) -> Self {
        Self(vec![value])
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
