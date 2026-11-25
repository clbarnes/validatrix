use crate::{
    accum::{Accumulator, Levels},
    errors::DiagnosticError,
};

/// Trait for synchronous validation.
pub trait Validate {
    /// Perform top-level validation on this value.
    ///
    /// Should not be called inside other validators;
    /// use [Validate::validate_inner] instead.
    /// Should not be overridden by implementors.
    fn validate(&self) -> crate::DiagnosticResult {
        self.validate_levels(Default::default())
    }

    /// Perform top-level validation on this value, failing at the first invalid item.
    ///
    /// Should not be called inside other validators;
    /// use [Validate::validate_inner] instead.
    /// Should not be overridden by implementors.
    fn validate_levels(&self, levels: Levels) -> crate::DiagnosticResult {
        let mut accum = Accumulator::with_levels(levels);
        self.validate_inner(&mut accum)?;
        accum.into()
    }

    /// Accumulate validation errors.
    ///
    /// Validators of containing types should call this;
    /// end users probably want [Validate::validate] instead.
    ///
    /// Should return an error (generally by `?`-propagating the results of [Accumulator] methods)
    /// as soon as the accumulator reports one, i.e. when it is not exhaustive.
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
