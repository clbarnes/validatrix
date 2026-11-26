use crate::diagnostics::Diagnostics;

/// Result of a top-level validation: non-fatal [Diagnostics] on success.
pub type DiagnosticResult<T = Diagnostics, E = DiagnosticError> = std::result::Result<T, E>;

const INDENT: &str = "   ";

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Validation diagnostics:")?;
        for fa in self.0.iter() {
            write!(f, "\n{INDENT}{fa}")?;
        }
        Ok(())
    }
}

impl std::error::Error for DiagnosticError {}

/// Validation error containing the [Diagnostics] collected before failing.
#[derive(Debug)]
pub struct DiagnosticError(pub(crate) Diagnostics);

impl From<DiagnosticError> for Diagnostics {
    fn from(err: DiagnosticError) -> Self {
        err.0
    }
}

impl DiagnosticError {
    /// Number of diagnostics.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// One step in the path to a validated value.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum Key {
    /// Position in a sequence.
    Index(usize),
    /// Name of a field or map key.
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
