#![doc=include_str!("../README.md")]
mod errors;
pub use errors::{Accumulator, Error, Failure, Key, Result};
pub mod synch;
pub use synch::{Validate, ValidateContext};
mod wrapper;
pub use wrapper::Valid;

pub mod asynch;
