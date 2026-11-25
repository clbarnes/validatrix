#![doc=include_str!("../README.md")]
mod accum;
mod errors;

pub use accum::Accumulator;
pub use errors::{Error, Failure, Key, Result};
pub mod traits;
pub use traits::Validate;
mod wrapper;
pub use wrapper::Valid;
pub(crate) mod diagnostics;
pub(crate) mod sharedbuf;
