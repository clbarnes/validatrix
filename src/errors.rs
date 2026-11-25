use std::fmt::{Display, Write};

use crate::{diagnostics::Diagnostics, Validate};

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
pub struct Error(Diagnostics);

impl Error {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<Accumulator> for Result<(), Error> {
    fn from(value: Accumulator) -> Self {
        if value.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(Error(value.diagnostics))
        }
    }
}

/// Type used to build up a list of validation failures.
///
/// When validating fields of a struct while validating that struct,
/// push the field's name (or index in a sequence)
/// onto the `prefix` member of the accumulator
/// before passing it in to the validator.
/// This allows nested fields to report where the failure happened.
///
/// ```
/// use validatrix::Accumulator;
///
/// fn accumulate(accum: &mut Accumulator) -> Result<(), validatrix::Error> {
///     accum.with_key("some_field", |a| {
///         if true == false {
///             a.add_failure("pigs have flown")?;
///         }
///         Ok(())
///     })?;
///
///     // equivalent to the above
///     if true == false {
///         accum.add_failure_at("some_field", "hell has frozen over")?;
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Accumulator {
    /// This prefix is applied to any failures added to the accumulator.
    prefix: Vec<Key>,
    diagnostics: Diagnostics,
    exhaustive: bool,
}

impl Accumulator {
    pub(crate) fn new() -> Self {
        Self {
            prefix: Default::default(),
            diagnostics: Default::default(),
            exhaustive: true,
        }
    }
}

impl Accumulator {
    pub(crate) fn exhaustive(&mut self, exhaustive: bool) {
        self.exhaustive = exhaustive;
    }

    /// Whether this accumulator collects all failures (`true`)
    /// or stops at the first one (`false`).
    pub fn is_exhaustive(&self) -> bool {
        self.exhaustive
    }

    /// Return all failures so far as an [Error]
    /// if this accumulator is not exhaustive and has any failures.
    pub fn fail_early(&self) -> Result<(), Error> {
        if !self.exhaustive && !self.diagnostics.is_empty() {
            Err(Error(self.diagnostics.clone()))
        } else {
            Ok(())
        }
    }

    /// Add an extra failure to this accumulator.
    ///
    /// Returns an error if this accumulator is not exhaustive.
    pub fn add_failure(&mut self, message: impl AsRef<str>) -> Result<(), Error> {
        self.diagnostics.push(&self.prefix, message);
        self.fail_early()
    }

    /// Accumulate an extra failure at the given key.
    ///
    /// Returns an error if this accumulator is not exhaustive.
    pub fn add_failure_at(
        &mut self,
        prefix: impl Into<Key>,
        message: impl AsRef<str>,
    ) -> Result<(), Error> {
        self.with_key(prefix, |a| a.add_failure(message))
    }

    /// Accumulate any validation errors for a [Validate] field with key `field`.
    ///
    /// Returns an error if this accumulator is not exhaustive and a failure occurred.
    pub fn validate_member_at(
        &mut self,
        field: impl Into<Key>,
        member: &impl Validate,
    ) -> Result<(), Error> {
        self.with_key(field, |a| member.validate_inner(a))
    }

    /// Like [Self::validate_member_at], but for a [crate::ValidateContext] field with the given context.
    pub fn validate_member_at_ctx<T: crate::ValidateContext>(
        &mut self,
        field: impl Into<Key>,
        member: &T,
        context: &T::Context,
    ) -> Result<(), Error> {
        self.with_key(field, |a| member.validate_inner_ctx(a, context))
    }

    /// Perform manual validation inside the given closure for a member with the given prefix.
    ///
    /// The closure takes an accumulator as an argument,
    /// which will be this accumulator with the added prefix.
    /// The prefix is removed again even if the closure returns an error.
    pub fn with_key(
        &mut self,
        prefix: impl Into<Key>,
        f: impl FnOnce(&mut Self) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.prefix.push(prefix.into());
        let res = f(self);
        self.prefix.pop();
        res?;
        self.fail_early()
    }

    /// Convenience method for [Accumulator::with_key]-like behaviour at multiple keys' depth.
    pub fn with_keys(
        &mut self,
        prefixes: &[Key],
        f: impl FnOnce(&mut Self) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let len = prefixes.len();
        self.prefix.extend_from_slice(prefixes);
        let res = f(self);
        self.prefix.truncate(self.prefix.len() - len);
        res?;
        self.fail_early()
    }

    /// Iterate over a collection of [Validate]-able items,
    /// validating them all in turn.
    /// As this tracks the items' index in the iterable,
    /// the whole collection should be passed rather than a filtered version.
    pub fn validate_iter<'a, V: Validate + 'a, I: IntoIterator<Item = &'a V>>(
        &mut self,
        items: I,
    ) -> Result<(), Error> {
        for (idx, item) in items.into_iter().enumerate() {
            self.validate_member_at(idx, item)?;
        }
        Ok(())
    }

    /// Like [Self::validate_iter], but for a collection of [crate::ValidateContext] items with the given context.
    pub fn validate_iter_ctx<'a, V: crate::ValidateContext + 'a, I: IntoIterator<Item = &'a V>>(
        &mut self,
        items: I,
        context: &V::Context,
    ) -> Result<(), Error> {
        for (idx, item) in items.into_iter().enumerate() {
            self.validate_member_at_ctx(idx, item, context)?;
        }
        Ok(())
    }

    /// Convenience method to do [Self::validate_iter] for a given key.
    pub fn validate_iter_at<'a, V: Validate + 'a, I: IntoIterator<Item = &'a V>>(
        &mut self,
        prefix: impl Into<Key>,
        items: I,
    ) -> Result<(), Error> {
        self.with_key(prefix, |a| a.validate_iter(items))
    }

    /// Like [Self::validate_iter_at], but for a collection of [crate::ValidateContext] items with the given context.
    pub fn validate_iter_at_ctx<
        'a,
        V: crate::ValidateContext + 'a,
        I: IntoIterator<Item = &'a V>,
    >(
        &mut self,
        prefix: impl Into<Key>,
        items: I,
        context: &V::Context,
    ) -> Result<(), Error> {
        self.with_key(prefix, |a| a.validate_iter_ctx(items, context))
    }

    /// Number of failures logged by this accumulator.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Whether this accumulator has 0 failures.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Struct representing a single validation failure.
/// Used to build informative error messages for [Error].
#[derive(Debug, Clone)]
pub struct Failure {
    pub(crate) key: Vec<Key>,
    // todo: replace with Cow?
    pub(crate) message: String,
}

impl Failure {
    pub fn new(path: &[Key], msg: impl Into<String>) -> Self {
        Self {
            key: path.to_vec(),
            message: msg.into(),
        }
    }
}

impl Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('$')?;
        for c in self.key.iter() {
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
