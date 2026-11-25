use crate::diagnostics::Diagnostics;

pub type DiagnosticResult<T = Diagnostics, E = DiagnosticError> = std::result::Result<T, E>;

const INDENT: &str = "   ";

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Validation diagnostics:")?;
        for fa in self.0.iter() {
            f.write_str("\n")?;
            f.write_str(INDENT)?;
            fa.fmt(f)?;
        }
        Ok(())
    }
}

impl std::error::Error for DiagnosticError {}

/// Validation error type wrapping [Diagnostics].
#[derive(Debug)]
pub struct DiagnosticError(pub(crate) Diagnostics);

impl DiagnosticError {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
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
