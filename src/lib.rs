#![doc=include_str!("../README.md")]
mod accum;
mod errors;

pub use accum::{Accumulator, Levels};
pub use errors::{DiagnosticError, DiagnosticResult, Key};
pub mod traits;
pub use traits::Validate;
mod wrapper;
pub use wrapper::{Valid, Validator};
pub(crate) mod diagnostics;
pub use diagnostics::{Diagnostic, Diagnostics, Level};
pub(crate) mod sharedbuf;
