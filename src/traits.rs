use crate::{accum::Accumulator, errors::Error};

/// Trait for synchronous validation.
pub trait Validate {
    /// Perform top-level validation on this value.
    ///
    /// Should not be called inside other validators;
    /// use [Validate::validate_inner] instead.
    /// Should not be overridden by implementors.
    fn validate(&self) -> crate::Result {
        let mut accum = Accumulator::new();
        // in exhaustive mode, failures are recorded in the accumulator rather than returned
        self.validate_inner(&mut accum)?;
        accum.into()
    }

    /// Perform top-level validation on this value, failing at the first invalid item.
    ///
    /// Should not be called inside other validators;
    /// use [Validate::validate_inner] instead.
    /// Should not be overridden by implementors.
    fn validate_fast(&self) -> crate::Result {
        let mut accum = Accumulator::new();
        accum.exhaustive(false);
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
    fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), Error>;
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
        fn validate_inner(&self, accum: &mut accum::Accumulator) -> Result<(), Error> {
            if self.avalue % 2 != 0 {
                accum.add_failure_at("avalue", "value is odd")?;
            }

            accum.validate_member_at("b", &self.b)
        }
    }

    struct B {
        bvalue: u8,
        cs: Vec<C>,
    }

    impl Validate for B {
        fn validate_inner(&self, accum: &mut accum::Accumulator) -> Result<(), Error> {
            if self.bvalue % 2 != 0 {
                accum.add_failure_at("bvalue", "value is odd")?;
            }

            accum.validate_iter_at("cs", &self.cs)
        }
    }

    struct C {
        cvalue: u8,
    }

    impl Validate for C {
        fn validate_inner(&self, accum: &mut accum::Accumulator) -> Result<(), Error> {
            if self.cvalue % 2 != 0 {
                accum.add_failure_at("cvalue", "value is odd")?;
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
