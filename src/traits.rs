use crate::{
    accum::{Accumulator, Policy},
    errors::DiagnosticError,
};

/// A type which can be validated.
pub trait Validate {
    /// Validate with the default [Policy].
    ///
    /// This should be used by consumers and not overridden by implementors
    /// (implementors should use [Validate::validate_inner]).
    fn validate(&self) -> crate::DiagnosticResult {
        self.validate_policy(Default::default())
    }

    /// Validate with the given [Policy].
    ///
    /// This should be used by consumers and not overridden by implementors
    /// (implementors should use [Validate::validate_inner]).
    fn validate_policy(&self, policy: Policy) -> crate::DiagnosticResult {
        let mut accum = Accumulator::with_policy(policy);
        self.validate_inner(&mut accum)?;
        accum.into()
    }

    /// Record diagnostics for this value.
    ///
    /// Propagate errors from [Accumulator] methods with `?`;
    /// this will obey the accumulator's [Policy].
    ///
    /// Consumers wanting to validate types defined elsewhere should not call this method:
    /// use [Self::validate_policy] instead.
    fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError>;
}

#[cfg(test)]
mod tests {
    use super::Validate;

    use crate::*;

    struct A {
        avalue: u8,
        b: B,
    }

    impl Validate for A {
        fn validate_inner(&self, accum: &mut accum::Accumulator) -> Result<(), DiagnosticError> {
            if self.avalue % 2 != 0 {
                accum.diagnose_at("avalue", Level::Error, "value is odd")?;
            }

            accum.validate_member_at("b", &self.b)
        }
    }

    struct B {
        bvalue: u8,
        cs: Vec<C>,
    }

    impl Validate for B {
        fn validate_inner(&self, accum: &mut accum::Accumulator) -> Result<(), DiagnosticError> {
            if self.bvalue % 2 != 0 {
                accum.diagnose_at("bvalue", Level::Error, "value is odd")?;
            }

            accum.validate_iter_at("cs", &self.cs)
        }
    }

    struct C {
        cvalue: u8,
    }

    impl Validate for C {
        fn validate_inner(&self, accum: &mut accum::Accumulator) -> Result<(), DiagnosticError> {
            if self.cvalue % 2 != 0 {
                accum.diagnose_at("cvalue", Level::Error, "value is odd")?;
            }
            Ok(())
        }
    }

    #[test]
    fn valid() {
        let valid = A {
            avalue: 0,
            b: B {
                bvalue: 0,
                cs: vec![C { cvalue: 0 }],
            },
        };
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn invalid() {
        let valid = A {
            avalue: 1,
            b: B {
                bvalue: 1,
                cs: vec![C { cvalue: 1 }, C { cvalue: 1 }],
            },
        };
        let err = valid.validate().unwrap_err();
        println!("{err}");
    }
}
