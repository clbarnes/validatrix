use std::fmt::{Display, Write};

use crate::{
    sharedbuf::{SharedBuf, SharedBufIter, SharedStringBuf, SharedStringBufIter},
    Key,
};

#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
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

    #[allow(unused)]
    pub fn push_prefix(&mut self, prefix: &[Key], path: &[Key], message: impl AsRef<str>) {
        self.paths
            .push(prefix.iter().copied().chain(path.iter().copied()));
        self.messages.push(message);
    }

    pub fn push(&mut self, path: &[Key], message: impl AsRef<str>) {
        self.paths.push(path.iter().copied());
        self.messages.push(message);
    }

    pub fn iter(&self) -> DiagnosticIter<'_> {
        DiagnosticIter {
            path_iter: self.paths.iter(),
            message_iter: self.messages.iter(),
        }
    }
}

pub struct DiagnosticIter<'a> {
    path_iter: SharedBufIter<'a, Key>,
    message_iter: SharedStringBufIter<'a>,
}

impl<'a> Iterator for DiagnosticIter<'a> {
    type Item = Diagnostic<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.path_iter.next()?;
        let message = self.message_iter.next()?;
        Some(Diagnostic { path, message })
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Diagnostic<'a> {
    path: &'a [Key],
    message: &'a str,
}

impl Display for Diagnostic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('$')?;
        for c in self.path.iter() {
            match c {
                Key::Index(n) => f.write_fmt(format_args!("[{n}]"))?,
                Key::Field(s) => {
                    f.write_char('.')?;
                    f.write_str(s)?;
                }
            }
        }
        f.write_str(": ")?;
        f.write_str(self.message)
    }
}
