#![doc=include_str!("../README.md")]
mod errors;
pub use errors::{Accumulator, Error, Failure, Result};
pub mod synch;
pub use synch::{Validate, ValidateContext};
mod wrapper;
pub use wrapper::Valid;

pub mod asynch;

#[cfg(test)]
mod tests {
    use crate::synch::Validate;

    use super::*;

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
}
