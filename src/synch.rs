use crate::errors::Accumulator;

/// Trait for synchronous validation.
pub trait Validate {
    /// Perform top-level validation on this value.
    ///
    /// Should not be called inside other validators;
    /// use [Validate::validate_inner] instead.
    /// Should not be overridden by implementors.
    fn validate(&self) -> crate::Result {
        let mut accum = Accumulator::new();
        self.validate_inner(&mut accum);
        accum.into()
    }

    /// Accumulate validation errors.
    ///
    /// Validators of containing types should call this;
    /// end users probably want [Validate::validate] instead.
    fn validate_inner(&self, accum: &mut Accumulator);
}

/// Trait for synchronous validation where some external data or context is required.
pub trait ValidateContext {
    /// Type of context which the validator needs (external data, resources etc.)
    type Context;

    /// Perform top-level validation on this value, with the given context.
    ///
    /// Should not be called inside other validators;
    /// use [ValidateContext::validate_inner] instead.
    /// Should not be overridden by implementors.
    fn validate_ctx(&self, context: &Self::Context) -> crate::Result {
        let mut accum = Accumulator::new();
        self.validate_inner_ctx(&mut accum, context);
        accum.into()
    }

    /// Accumulate validation errors.
    ///
    /// Validators of containing types should call this;
    /// end users probably want [ValidateContext::validate] instead.
    fn validate_inner_ctx(&self, accum: &mut Accumulator, context: &Self::Context);
}

#[cfg(test)]
mod tests {
    use super::{Validate, ValidateContext};

    use crate::*;

    struct A {
        avalue: u8,
        b: B,
    }

    impl Validate for A {
        fn validate_inner(&self, accum: &mut errors::Accumulator) {
            if self.avalue % 2 != 0 {
                accum.add_failure_at("avalue", "value is odd");
            }

            accum.validate_member_at("b", &self.b);
        }
    }

    struct B {
        bvalue: u8,
        cs: Vec<C>,
    }

    impl Validate for B {
        fn validate_inner(&self, accum: &mut errors::Accumulator) {
            if self.bvalue % 2 != 0 {
                accum.add_failure_at("bvalue", "value is odd");
            }

            accum.validate_iter_at("cs", &self.cs);
        }
    }

    struct C {
        cvalue: u8,
    }

    impl Validate for C {
        fn validate_inner(&self, accum: &mut errors::Accumulator) {
            if self.cvalue % 2 != 0 {
                accum.add_failure_at("cvalue", "value is odd");
            }
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

    struct DContext {
        threshold: u8,
    }

    struct D(Vec<u8>);

    impl ValidateContext for D {
        type Context = DContext;

        fn validate_inner_ctx(&self, accum: &mut errors::Accumulator, context: &Self::Context) {
            for (i, v) in self.0.iter().enumerate() {
                if *v > context.threshold {
                    accum.add_failure_at(
                        i,
                        format!("value {v} exceeds threshold {}", context.threshold),
                    );
                }
            }
        }
    }

    #[test]
    fn valid_ctx() {
        let ctx = DContext { threshold: 3 };
        let d_valid = D(vec![0, 1, 2]);
        assert!(d_valid.validate_ctx(&ctx).is_ok());

        let d_invalid = D(vec![0, 1, 4]);
        assert!(d_invalid.validate_ctx(&ctx).is_err());
    }
}
