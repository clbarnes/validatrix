use crate::{
    diagnostics::Diagnostics,
    errors::{DiagnosticError, Key},
    Level, Validate,
};

/// Type used to build up a list of validation failures.
///
/// When validating fields of a struct while validating that struct,
/// push the field's name (or index in a sequence)
/// onto the `prefix` member of the accumulator
/// before passing it in to the validator.
/// This allows nested fields to report where the failure happened.
///
/// ```
/// use validatrix::{Accumulator, Level, DiagnosticError};
///
/// fn accumulate(accum: &mut Accumulator) -> Result<(), DiagnosticError> {
///     accum.with_key("some_field", |a| {
///         if true == false {
///             a.diagnose(Level::Error, "pigs have flown")?;
///         }
///         Ok(())
///     })?;
///
///     // equivalent to the above
///     if true == false {
///         accum.diagnose_at("some_field", Level::Error, "hell has frozen over")?;
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Default)]
pub struct Accumulator {
    /// This prefix is applied to any failures added to the accumulator.
    prefix: Vec<Key>,
    diagnostics: Diagnostics,
    levels: Levels,
}

/// Levels of validation diagnostics to ignore, raise later, or raise immediately.
///
/// Some common configurations are provided as associated constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Levels {
    /// Do not store diagnostics below this level (exclusive).
    pub ignore_below: Level,
    /// At the end of validation, raise an error if any diagnostics were above this level (exclusive).
    pub raise_above: Level,
    /// Raise an error immediately on a diagnostic above this level (exclusive).
    pub fail_fast_above: Level,
}

impl Default for Levels {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Levels {
    /// Ignore anything below Warning, raise Errors, and fail fast on Critical.
    pub const DEFAULT: Self = Self {
        ignore_below: Level::Warning,
        raise_above: Level::Warning,
        fail_fast_above: Level::Error,
    };
    /// Fail fast on Error and above.
    pub const FAST: Self = Self {
        ignore_below: Level::Warning,
        raise_above: Level::Warning,
        fail_fast_above: Level::Warning,
    };
    /// Only store errors.
    pub const QUIET: Self = Self {
        ignore_below: Level::Error,
        raise_above: Level::Warning,
        fail_fast_above: Level::Error,
    };
    /// Store everything, error on warnings.
    pub const STRICT: Self = Self {
        ignore_below: Level::MIN,
        raise_above: Level::Info,
        fail_fast_above: Level::Error,
    };

    pub fn new(ignore_below: Level, raise_above: Level, fail_fast_above: Level) -> Self {
        Self {
            ignore_below,
            raise_above,
            fail_fast_above,
        }
    }
}

impl Accumulator {
    pub(crate) fn with_levels(levels: Levels) -> Self {
        Self {
            prefix: Default::default(),
            diagnostics: Default::default(),
            levels,
        }
    }

    pub fn diagnose(
        &mut self,
        level: Level,
        message: impl AsRef<str>,
    ) -> Result<(), DiagnosticError> {
        if level < self.levels.ignore_below {
            return Ok(());
        }
        self.diagnostics.push(&self.prefix, level, message);
        if level > self.levels.fail_fast_above {
            let diag = std::mem::take(&mut self.diagnostics);
            Err(DiagnosticError(diag))
        } else {
            Ok(())
        }
    }

    pub fn diagnose_at(
        &mut self,
        prefix: impl Into<Key>,
        level: Level,
        message: impl AsRef<str>,
    ) -> Result<(), DiagnosticError> {
        self.with_key(prefix, |a| a.diagnose(level, message))
    }

    /// Add an extra failure to this accumulator.
    ///
    /// Returns an error if the level is above the accumulator's `fail_fast_above`.
    #[deprecated(note = "use `Self::diagnose` with Level::Error")]
    pub fn add_failure(&mut self, message: impl AsRef<str>) -> Result<(), DiagnosticError> {
        self.diagnose(Level::Error, message)
    }

    /// Accumulate an extra failure at the given key.
    ///
    /// Returns an error if the level is above the accumulator's `fail_fast_above`.
    #[deprecated(note = "use `Self::diagnose_at` with Level::Error")]
    pub fn add_failure_at(
        &mut self,
        prefix: impl Into<Key>,
        message: impl AsRef<str>,
    ) -> Result<(), DiagnosticError> {
        self.with_key(prefix, |a| a.diagnose(Level::Error, message))
    }

    /// Accumulate any validation errors for a [Validate] field with key `field`.
    ///
    /// Returns an error if this accumulator is not exhaustive and a failure occurred.
    pub fn validate_member_at(
        &mut self,
        field: impl Into<Key>,
        member: &impl Validate,
    ) -> Result<(), DiagnosticError> {
        self.with_key(field, |a| member.validate_inner(a))
    }

    /// Perform manual validation inside the given closure for a member with the given prefix.
    ///
    /// The closure takes an accumulator as an argument,
    /// which will be this accumulator with the added prefix.
    /// The prefix is removed again even if the closure returns an error.
    pub fn with_key(
        &mut self,
        prefix: impl Into<Key>,
        f: impl FnOnce(&mut Self) -> Result<(), DiagnosticError>,
    ) -> Result<(), DiagnosticError> {
        self.prefix.push(prefix.into());
        let res = f(self);
        self.prefix.pop();
        res
    }

    /// Convenience method for [Accumulator::with_key]-like behaviour at multiple keys' depth.
    pub fn with_keys(
        &mut self,
        prefixes: &[Key],
        f: impl FnOnce(&mut Self) -> Result<(), DiagnosticError>,
    ) -> Result<(), DiagnosticError> {
        let len = prefixes.len();
        self.prefix.extend_from_slice(prefixes);
        let res = f(self);
        self.prefix.truncate(self.prefix.len() - len);
        res
    }

    /// Iterate over a collection of [Validate]-able items,
    /// validating them all in turn.
    /// As this tracks the items' index in the iterable,
    /// the whole collection should be passed rather than a filtered version.
    pub fn validate_iter<'a, V: Validate + 'a, I: IntoIterator<Item = &'a V>>(
        &mut self,
        items: I,
    ) -> Result<(), DiagnosticError> {
        for (idx, item) in items.into_iter().enumerate() {
            self.validate_member_at(idx, item)?;
        }
        Ok(())
    }

    /// Convenience method to do [Self::validate_iter] for a given key.
    pub fn validate_iter_at<'a, V: Validate + 'a, I: IntoIterator<Item = &'a V>>(
        &mut self,
        prefix: impl Into<Key>,
        items: I,
    ) -> Result<(), DiagnosticError> {
        self.with_key(prefix, |a| a.validate_iter(items))
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

impl From<Accumulator> for crate::DiagnosticResult {
    fn from(accum: Accumulator) -> Self {
        accum.diagnostics.result_for_level(accum.levels.raise_above)
    }
}
