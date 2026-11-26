#![doc=include_str!("../README.md")]
mod accum;
mod errors;

pub use accum::{Accumulator, Policy};
pub use errors::{DiagnosticError, DiagnosticResult, Key};
pub mod traits;
pub use traits::Validate;
mod wrapper;
pub use wrapper::{Valid, Validator};
pub(crate) mod diagnostics;
pub use diagnostics::{Diagnostic, Diagnostics, Level};
pub(crate) mod sharedbuf;

/// Convenience module re-exporting items needed to validate values.
pub mod consume {
    pub use super::{Level, Policy, Valid, Validate, Validator};
}

/// Convenience module re-exporting items needed to implement [Validate].
pub mod implement {
    pub use super::{Accumulator, DiagnosticError, Level, Validate};
}
