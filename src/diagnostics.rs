use std::fmt::{Display, Write};

use crate::{
    sharedbuf::{SharedBuf, SharedBufIter, SharedStringBuf, SharedStringBufIter},
    Key,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Level {
    Info,
    Warning,
    #[default]
    Error,
    Critical,
}

impl Level {
    pub const MIN: Self = Self::Info;
    pub const MAX: Self = Self::Critical;

    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Info => "INFO",
            Level::Warning => "WARNING",
            Level::Error => "ERROR",
            Level::Critical => "CRITICAL",
        }
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    levels: Vec<Level>,
    paths: SharedBuf<Key>,
    messages: SharedStringBuf,
}

impl Diagnostics {
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub(crate) fn result_for_level(self, level: Level) -> crate::DiagnosticResult {
        if self.levels.iter().any(|&l| l > level) {
            Err(crate::DiagnosticError(self))
        } else {
            Ok(self)
        }
    }

    #[allow(unused)]
    pub fn push_prefix(
        &mut self,
        prefix: &[Key],
        path: &[Key],
        level: Level,
        message: impl AsRef<str>,
    ) {
        self.levels.push(level);
        self.paths
            .push(prefix.iter().copied().chain(path.iter().copied()));
        self.messages.push(message);
    }

    pub fn push(&mut self, path: &[Key], level: Level, message: impl AsRef<str>) {
        self.levels.push(level);
        self.paths.push(path.iter().copied());
        self.messages.push(message);
    }

    pub fn iter(&self) -> DiagnosticIter<'_> {
        DiagnosticIter {
            level_iter: self.levels.iter(),
            path_iter: self.paths.iter(),
            message_iter: self.messages.iter(),
        }
    }
}

pub struct DiagnosticIter<'a> {
    level_iter: std::slice::Iter<'a, Level>,
    path_iter: SharedBufIter<'a, Key>,
    message_iter: SharedStringBufIter<'a>,
}

impl<'a> Iterator for DiagnosticIter<'a> {
    type Item = Diagnostic<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.path_iter.next()?;
        let message = self.message_iter.next()?;
        Some(Diagnostic {
            level: *self.level_iter.next()?,
            path: path.into(),
            message,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct DiagnosticPath<'a> {
    path: &'a [Key],
}

impl<'a> From<&'a [Key]> for DiagnosticPath<'a> {
    fn from(path: &'a [Key]) -> Self {
        Self { path }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Diagnostic<'a> {
    pub level: Level,
    pub path: DiagnosticPath<'a>,
    pub message: &'a str,
}

impl Display for DiagnosticPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_jsonpath(self.path, f)
    }
}

fn fmt_jsonpath(path: &[Key], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_char('$')?;
    for k in path.iter() {
        fmt_key_jsonpath(k, f)?;
    }
    Ok(())
}

#[allow(unused)]
fn fmt_key_jsonpointer(key: &Key, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_char('/')?;
    match key {
        Key::Index(n) => f.write_fmt(format_args!("{n}")),
        Key::Field(s) => {
            for c in s.chars() {
                match c {
                    '~' => f.write_str("~0")?,
                    '/' => f.write_str("~1")?,
                    _ => f.write_char(c)?,
                }
            }
            Ok(())
        }
    }
}

fn fmt_key_jsonpath(key: &Key, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match key {
        Key::Index(n) => f.write_fmt(format_args!("[{n}]")),
        Key::Field(s) => {
            if is_jsonpath_shorthand(s) {
                f.write_char('.')?;
                f.write_str(s)
            } else {
                f.write_str("['")?;
                for c in s.chars() {
                    match c {
                        '\'' => f.write_str("\\'")?,
                        '\\' => f.write_str("\\\\")?,
                        _ => f.write_char(c)?,
                    }
                }
                f.write_str("']")
            }
        }
    }
}

/// Whether a field name can use the dot shorthand (`.name`) in JSONPath,
/// i.e. it is non-empty, contains only ASCII alphanumerics/underscores,
/// and does not start with a digit.
fn is_jsonpath_shorthand(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

impl Display for Diagnostic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}: {}", self.level, self.path, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct JsonPath<'a>(&'a [Key]);

    impl Display for JsonPath<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            fmt_jsonpath(self.0, f)
        }
    }

    #[test]
    fn jsonpath_keys() {
        let path = [
            Key::Field("plain_key1"),
            Key::Index(3),
            Key::Field("a.b"),
            Key::Field("it's"),
            Key::Field("back\\slash"),
            Key::Field("1st"),
            Key::Field(""),
        ];
        assert_eq!(
            JsonPath(&path).to_string(),
            r"$.plain_key1[3]['a.b']['it\'s']['back\\slash']['1st']['']"
        );
    }
}
