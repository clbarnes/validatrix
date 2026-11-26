use std::fmt::{Display, Write};

use crate::{
    sharedbuf::{SharedBuf, SharedBufIter, SharedStringBuf, SharedStringBufIter},
    Key,
};

/// Severity of a [Diagnostic], from least to most severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Level {
    /// Informational; not a problem.
    Info,
    /// Possible problem; the value is still usable.
    /// Deprecations should be at this level.
    Warning,
    /// The value is invalid.
    #[default]
    Error,
    /// The value is invalid and validation should not continue.
    Critical,
}

impl Level {
    /// Least severe level.
    pub const MIN: Self = Self::Info;
    /// Most severe level.
    pub const MAX: Self = Self::Critical;

    /// Upper-case name, e.g. `"ERROR"`.
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

/// Collection of [Diagnostic]s produced by validation.
#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    levels: Vec<Level>,
    paths: SharedBuf<Key>,
    messages: SharedStringBuf,
}

impl Diagnostics {
    /// Number of diagnostics.
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Whether there are no diagnostics.
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub(crate) fn result_for_level(self, level: Level) -> crate::DiagnosticResult {
        if self.levels.iter().any(|&l| l >= level) {
            Err(crate::DiagnosticError(self))
        } else {
            Ok(self)
        }
    }

    /// Add a diagnostic at `prefix` followed by `path`.
    #[allow(unused)]
    pub fn push_prefix(
        &mut self,
        prefix: &[Key],
        path: &[Key],
        level: Level,
        message: impl std::fmt::Display,
    ) {
        self.levels.push(level);
        self.paths
            .push(prefix.iter().copied().chain(path.iter().copied()));
        self.messages.push(message);
    }

    /// Add a diagnostic.
    pub fn push(&mut self, path: &[Key], level: Level, message: impl std::fmt::Display) {
        self.levels.push(level);
        self.paths.push(path.iter().copied());
        self.messages.push(message);
    }

    /// Iterate over diagnostics in the order they were added.
    pub fn iter(&self) -> DiagnosticIter<'_> {
        DiagnosticIter {
            level_iter: self.levels.iter(),
            path_iter: self.paths.iter(),
            message_iter: self.messages.iter(),
        }
    }
}

/// Iterator over [Diagnostics].
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

/// Location of a [Diagnostic]; displayed as a JSONPath, e.g. `$.items[0].name`.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct DiagnosticPath<'a> {
    path: &'a [Key],
}

impl<'a> From<&'a [Key]> for DiagnosticPath<'a> {
    fn from(path: &'a [Key]) -> Self {
        Self { path }
    }
}

/// A single validation message.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Diagnostic<'a> {
    /// Severity.
    pub level: Level,
    /// Location of the value the message refers to.
    pub path: DiagnosticPath<'a>,
    /// Human-readable description.
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
