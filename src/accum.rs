use crate::{
    diagnostics::Diagnostics,
    errors::{DiagnosticError, Key},
    Level, Validate,
};

/// Collects [Diagnostic](crate::Diagnostic)s while validating, tracking where each one occurred.
///
/// Methods return an error when validation should stop early (according to the [Policy]);
/// propagate it with `?`.
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
///
/// When diagnostics are added, they can be either ignored, collected, or cause an immediate error depending on the [Policy].
/// Accumulator methods should always propagate errors with `?`,
/// as those errors are only raised when the policy demands early termination.
#[derive(Debug)]
pub struct Accumulator {
    /// This prefix is applied to any failures added to the accumulator.
    prefix: Vec<Key>,
    diagnostics: Diagnostics,
    policy: Policy,
}

/// Which diagnostic [Level]s to ignore, return as errors at the end, or fail on immediately.
///
/// All thresholds are inclusive.
/// Common configurations are provided as associated constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    /// Discard diagnostics below this level.
    pub collect: Level,
    /// Diagnostics at this level or above should fail validation.
    pub fail: Level,
    /// Diagnostics at this level or above should fail validation immediately,
    /// skipping all further validation.
    pub abort: Level,
}

impl Default for Policy {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Policy {
    /// Ignore info, fail on errors, fail fast on critical.
    pub const DEFAULT: Self = Self {
        collect: Level::Warning,
        fail: Level::Error,
        abort: Level::MAX,
    };
    /// Like [Self::DEFAULT], but fail fast on errors.
    pub const FAST: Self = Self {
        collect: Level::Warning,
        fail: Level::Error,
        abort: Level::Error,
    };
    /// Like [Self::DEFAULT], but only keep errors and above.
    pub const QUIET: Self = Self {
        collect: Level::Error,
        fail: Level::Error,
        abort: Level::MAX,
    };
    /// Like [Self::DEFAULT], but keep everything.
    pub const DETAILED: Self = Self {
        collect: Level::MIN,
        fail: Level::Error,
        abort: Level::MAX,
    };
    /// Keep everything; fail on warnings.
    pub const STRICT: Self = Self {
        collect: Level::MIN,
        fail: Level::Warning,
        abort: Level::MAX,
    };

    /// See the field docs.
    pub fn new(collect: Level, fail: Level, abort: Level) -> Self {
        Self {
            collect,
            fail,
            abort,
        }
    }

    /// Whether a diagnostic at the given level will
    /// - be collected
    /// - cause an eventual error
    /// - cause an immediate error
    pub fn for_level(&self, level: Level) -> (bool, bool, bool) {
        (
            level >= self.collect,
            level >= self.fail,
            level >= self.abort,
        )
    }
}

impl Accumulator {
    pub(crate) fn with_policy(policy: Policy) -> Self {
        Self {
            prefix: Default::default(),
            diagnostics: Default::default(),
            policy,
        }
    }

    /// The ignore/ collect/ fail-fast thresholds for this accumulator.
    pub fn policy(&self) -> &Policy {
        &self.policy
    }

    /// The current prefix for diagnostics.
    pub fn prefix(&self) -> &[Key] {
        &self.prefix
    }

    /// Record a diagnostic at the current location.
    ///
    /// This may be ignored, collected, or cause an immediate error depending on the [Policy].
    pub fn diagnose(
        &mut self,
        level: Level,
        message: impl std::fmt::Display,
    ) -> Result<(), DiagnosticError> {
        if level < self.policy.collect {
            return Ok(());
        }
        self.diagnostics.push(&self.prefix, level, message);
        if level >= self.policy.abort {
            let diag = std::mem::take(&mut self.diagnostics);
            Err(DiagnosticError(diag))
        } else {
            Ok(())
        }
    }

    /// Record a diagnostic at the given child key.
    ///
    /// This may be ignored, collected, or cause an immediate error depending on the [Policy].
    pub fn diagnose_at(
        &mut self,
        prefix: impl Into<Key>,
        level: Level,
        message: impl std::fmt::Display,
    ) -> Result<(), DiagnosticError> {
        self.with_key(prefix, |a| a.diagnose(level, message))
    }

    /// DEPRECATED: Record an error at the current location.
    #[deprecated(note = "use `Self::diagnose` with Level::Error")]
    pub fn add_failure(&mut self, message: impl AsRef<str>) -> Result<(), DiagnosticError> {
        self.diagnose(Level::Error, message.as_ref())
    }

    /// DEPRECATED: Record an error at the given child key.
    #[deprecated(note = "use `Self::diagnose_at` with Level::Error")]
    pub fn add_failure_at(
        &mut self,
        prefix: impl Into<Key>,
        message: impl AsRef<str>,
    ) -> Result<(), DiagnosticError> {
        self.with_key(prefix, |a| a.diagnose(Level::Error, message.as_ref()))
    }

    /// Validate a child value at the given key.
    pub fn validate_member_at(
        &mut self,
        field: impl Into<Key>,
        member: &impl Validate,
    ) -> Result<(), DiagnosticError> {
        self.with_key(field, |a| member.validate_inner(a))
    }

    /// Run custom validation at the given child key.
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

    /// Like [Self::with_key], for a nested path.
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

    /// Validate each item, keyed by its index.
    ///
    /// Pass the whole collection (not a filtered one) so indices are correct.
    pub fn validate_iter<'a, V: Validate + 'a, I: IntoIterator<Item = &'a V>>(
        &mut self,
        items: I,
    ) -> Result<(), DiagnosticError> {
        for (idx, item) in items.into_iter().enumerate() {
            self.validate_member_at(idx, item)?;
        }
        Ok(())
    }

    /// [Self::validate_iter] at the given child key.
    pub fn validate_iter_at<'a, V: Validate + 'a, I: IntoIterator<Item = &'a V>>(
        &mut self,
        prefix: impl Into<Key>,
        items: I,
    ) -> Result<(), DiagnosticError> {
        self.with_key(prefix, |a| a.validate_iter(items))
    }

    /// Number of diagnostics recorded.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Whether no diagnostics have been recorded.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

impl From<Accumulator> for crate::DiagnosticResult {
    fn from(accum: Accumulator) -> Self {
        accum.diagnostics.result_for_level(accum.policy.fail)
    }
}
