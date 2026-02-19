use std::fmt::{Display, Write};

use crate::Validate;

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
pub struct Error(Vec<Failure>);

impl Error {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<Accumulator> for Result<(), Error> {
    fn from(value: Accumulator) -> Self {
        if value.failures.is_empty() {
            Ok(())
        } else {
            Err(Error(value.failures))
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
/// fn accumulate(accum: &mut Accumulator) {
///     accum.with_key("some_field", |a| if true == false { a.add_failure("pigs have flown") });
///
///     // equivalent to the above
///     if true == false {
///         accum.add_failure_at("some_field", "hell has frozen over");
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Accumulator {
    /// This prefix is applied to any failures added to the accumulator.
    prefix: Vec<Key>,
    failures: Vec<Failure>,
}

impl Accumulator {
    pub(crate) fn new() -> Self {
        Self {
            prefix: Default::default(),
            failures: Default::default(),
        }
    }
}

impl Accumulator {
    /// Add an extra failure to this accumulator.
    pub fn add_failure(&mut self, message: impl Into<String>) {
        self.failures.push(Failure::new(&self.prefix, message))
    }

    /// Accumulate an extra failure at the given key.
    pub fn add_failure_at(&mut self, prefix: impl Into<Key>, message: impl Into<String>) {
        self.with_key(prefix, |a| a.add_failure(message))
    }

    /// Accumulate any validation errors for a [Validate] field with key `field`.
    pub fn validate_member_at(&mut self, field: impl Into<Key>, member: &impl Validate) {
        self.with_key(field, |a| member.validate_inner(a))
    }

    /// Like [Self::validate_member_at], but for a [crate::ValidateContext] field with the given context.
    pub fn validate_member_at_ctx<T: crate::ValidateContext>(
        &mut self,
        field: impl Into<Key>,
        member: &T,
        context: &T::Context,
    ) {
        self.with_key(field, |a| member.validate_inner_ctx(a, context))
    }

    /// Perform manual validation inside the given closure for a member with the given prefix.
    ///
    /// The closure takes an accumulator as an argument,
    /// which will be this accumulator with the added prefix.
    pub fn with_key(&mut self, prefix: impl Into<Key>, f: impl FnOnce(&mut Self)) {
        self.prefix.push(prefix.into());
        f(self);
        self.prefix.pop();
    }

    /// Convenience method for [Accumulator::with_key]-like behaviour at multiple keys' depth.
    pub fn with_keys(&mut self, prefixes: &[Key], f: impl FnOnce(&mut Self)) {
        let len = prefixes.len();
        for p in prefixes {
            self.prefix.push(*p);
        }
        f(self);
        for _ in 0..len {
            self.prefix.pop();
        }
    }

    /// Iterate over a collection of [Validate]-able items,
    /// validating them all in turn.
    /// As this tracks the items' index in the iterable,
    /// the whole collection should be passed rather than a filtered version.
    pub fn validate_iter<'a, V: Validate + 'a, I: IntoIterator<Item = &'a V>>(&mut self, items: I) {
        items.into_iter().enumerate().for_each(|(idx, item)| {
            self.validate_member_at(idx, item);
        })
    }

    /// Like [Self::validate_iter], but for a collection of [crate::ValidateContext] items with the given context.
    pub fn validate_iter_ctx<'a, V: crate::ValidateContext + 'a, I: IntoIterator<Item = &'a V>>(
        &mut self,
        items: I,
        context: &V::Context,
    ) {
        items.into_iter().enumerate().for_each(|(idx, item)| {
            self.validate_member_at_ctx(idx, item, context);
        })
    }

    /// Convenience method to do [Self::validate_iter] for a given key.
    pub fn validate_iter_at<'a, V: Validate + 'a, I: IntoIterator<Item = &'a V>>(
        &mut self,
        prefix: impl Into<Key>,
        items: I,
    ) {
        self.with_key(prefix, |a| a.validate_iter(items));
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
    ) {
        self.with_key(prefix, |a| a.validate_iter_ctx(items, context));
    }

    /// Number of failures logged by this accumulator.
    pub fn len(&self) -> usize {
        self.failures.len()
    }

    /// Whether this accumulator has 0 failures.
    pub fn is_empty(&self) -> bool {
        self.failures.is_empty()
    }
}

/// Struct representing a single validation failure.
/// Used to build informative error messages for [Error].
#[derive(Debug)]
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

impl<T: Into<String>> From<T> for Failure {
    fn from(value: T) -> Self {
        Self {
            key: Default::default(),
            message: value.into(),
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

impl From<Failure> for Error {
    fn from(value: Failure) -> Self {
        Self(vec![value])
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
