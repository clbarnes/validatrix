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
///
/// Messages are formatted into a shared buffer with [std::fmt::Display]; if you can't use a `&str` message, prefer [format_args] rather than [format] to save the allocation.
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

#[cfg(test)]
mod tests {
    use super::*;

    const LEVELS: [Level; 4] = [Level::Info, Level::Warning, Level::Error, Level::Critical];

    /// What happened to a single diagnostic.
    #[derive(Debug, PartialEq, Eq)]
    struct Outcome {
        collected: bool,
        failed: bool,
        aborted: bool,
    }

    /// Run one diagnostic at `level` through the full validate path.
    fn outcome(policy: Policy, level: Level) -> Outcome {
        struct One(Level);
        impl Validate for One {
            fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError> {
                accum.diagnose(self.0, "msg")
            }
        }

        let mut accum = Accumulator::with_policy(policy);
        let aborted = accum.diagnose(level, "msg").is_err();
        let collected = aborted || accum.len() == 1;
        let failed = One(level).validate_policy(policy).is_err();
        Outcome {
            collected,
            failed,
            aborted,
        }
    }

    fn levels_of(diags: &Diagnostics) -> Vec<Level> {
        diags.iter().map(|d| d.level).collect()
    }

    #[test]
    fn default_policy() {
        let p = Policy::DEFAULT;
        let o = |c, f, a| Outcome {
            collected: c,
            failed: f,
            aborted: a,
        };
        assert_eq!(outcome(p, Level::Info), o(false, false, false));
        assert_eq!(outcome(p, Level::Warning), o(true, false, false));
        assert_eq!(outcome(p, Level::Error), o(true, true, false));
        assert_eq!(outcome(p, Level::Critical), o(true, true, true));
    }

    #[test]
    fn fast_policy() {
        let p = Policy::FAST;
        let o = |c, f, a| Outcome {
            collected: c,
            failed: f,
            aborted: a,
        };
        assert_eq!(outcome(p, Level::Info), o(false, false, false));
        assert_eq!(outcome(p, Level::Warning), o(true, false, false));
        assert_eq!(outcome(p, Level::Error), o(true, true, true));
        assert_eq!(outcome(p, Level::Critical), o(true, true, true));
    }

    #[test]
    fn quiet_policy() {
        let p = Policy::QUIET;
        let o = |c, f, a| Outcome {
            collected: c,
            failed: f,
            aborted: a,
        };
        assert_eq!(outcome(p, Level::Info), o(false, false, false));
        assert_eq!(outcome(p, Level::Warning), o(false, false, false));
        assert_eq!(outcome(p, Level::Error), o(true, true, false));
        assert_eq!(outcome(p, Level::Critical), o(true, true, true));
    }

    #[test]
    fn detailed_policy() {
        let p = Policy::DETAILED;
        let o = |c, f, a| Outcome {
            collected: c,
            failed: f,
            aborted: a,
        };
        assert_eq!(outcome(p, Level::Info), o(true, false, false));
        assert_eq!(outcome(p, Level::Warning), o(true, false, false));
        assert_eq!(outcome(p, Level::Error), o(true, true, false));
        assert_eq!(outcome(p, Level::Critical), o(true, true, true));
    }

    #[test]
    fn strict_policy() {
        let p = Policy::STRICT;
        let o = |c, f, a| Outcome {
            collected: c,
            failed: f,
            aborted: a,
        };
        assert_eq!(outcome(p, Level::Info), o(true, false, false));
        assert_eq!(outcome(p, Level::Warning), o(true, true, false));
        assert_eq!(outcome(p, Level::Error), o(true, true, false));
        assert_eq!(outcome(p, Level::Critical), o(true, true, true));
    }

    /// Every combination of thresholds and diagnostic levels.
    /// A diagnostic below the collection threshold is ignored entirely,
    /// even if it would otherwise fail or abort.
    /// Aborting always fails validation, even if `abort` is below `fail`.
    #[test]
    fn all_thresholds() {
        for collect in LEVELS {
            for fail in LEVELS {
                for abort in LEVELS {
                    let policy = Policy::new(collect, fail, abort);
                    for level in LEVELS {
                        let collected = level >= collect;
                        let aborted = collected && level >= abort;
                        let expected = Outcome {
                            collected,
                            failed: aborted || (collected && level >= fail),
                            aborted,
                        };
                        assert_eq!(
                            outcome(policy, level),
                            expected,
                            "{policy:?} with {level:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn abort_returns_earlier_collected_diagnostics() {
        let mut accum = Accumulator::with_policy(Policy::FAST);
        accum.diagnose(Level::Info, "ignored").unwrap();
        accum.diagnose(Level::Warning, "kept").unwrap();
        let err = accum.diagnose(Level::Error, "abort").unwrap_err();
        assert_eq!(levels_of(&err.0), [Level::Warning, Level::Error]);
    }

    #[test]
    fn abort_skips_later_validation() {
        struct Two;
        impl Validate for Two {
            fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError> {
                accum.diagnose_at("a", Level::Error, "first")?;
                accum.diagnose_at("b", Level::Error, "second")
            }
        }

        let err = Two.validate_policy(Policy::FAST).unwrap_err();
        assert_eq!(err.len(), 1);

        let err = Two.validate_policy(Policy::DEFAULT).unwrap_err();
        assert_eq!(err.len(), 2);
    }

    #[test]
    fn success_returns_collected_diagnostics() {
        struct Mixed;
        impl Validate for Mixed {
            fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError> {
                accum.diagnose(Level::Info, "info")?;
                accum.diagnose(Level::Warning, "warning")
            }
        }

        let diags = Mixed.validate_policy(Policy::DEFAULT).unwrap();
        assert_eq!(levels_of(&diags), [Level::Warning]);

        let diags = Mixed.validate_policy(Policy::DETAILED).unwrap();
        assert_eq!(levels_of(&diags), [Level::Info, Level::Warning]);

        let err = Mixed.validate_policy(Policy::STRICT).unwrap_err();
        assert_eq!(levels_of(&err.0), [Level::Info, Level::Warning]);
    }

    #[test]
    fn for_level_matches_behaviour() {
        for policy in [
            Policy::DEFAULT,
            Policy::FAST,
            Policy::QUIET,
            Policy::DETAILED,
            Policy::STRICT,
        ] {
            for level in LEVELS {
                let (c, f, a) = policy.for_level(level);
                let o = outcome(policy, level);
                assert_eq!(
                    (c, c && f, c && a),
                    (o.collected, o.failed, o.aborted),
                    "{policy:?} with {level:?}"
                );
            }
        }
    }
}
