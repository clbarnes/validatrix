mod errors;
pub use errors::{Accumulator, Error, Failure, Result};
mod sync;
pub use sync::Validate;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::sync::Validate;

    use super::*;

    struct A {
        avalue: u8,
        b: B,
    }

    impl Validate for A {
        fn validate_inner(&self, accum: &mut errors::Accumulator) -> usize {
            let orig = accum.len();
            if self.avalue % 2 != 0 {
                accum.add_failure("value is odd".into(), &["avalue".into()]);
            }

            accum.prefix.push("b".into());
            self.b.validate_inner(accum);
            accum.prefix.pop();

            accum.len() - orig
        }
    }

    struct B {
        bvalue: u8,
        cs: Vec<C>,
    }

    impl Validate for B {
        fn validate_inner(&self, accum: &mut errors::Accumulator) -> usize {
            let orig_len = accum.len();
            if self.bvalue % 2 != 0 {
                accum.add_failure("value is odd".into(), &["bvalue".into()]);
            }

            accum.prefix.push("cs".into());
            for (idx, c) in self.cs.iter().enumerate() {
                accum.prefix.push(idx.into());
                c.validate_inner(accum);
                accum.prefix.pop();
            }
            accum.prefix.pop();

            accum.len() - orig_len
        }
    }

    struct C {
        cvalue: u8,
    }

    impl Validate for C {
        fn validate_inner(&self, accum: &mut errors::Accumulator) -> usize {
            let orig_len = accum.len();
            if self.cvalue % 2 != 0 {
                accum.add_failure("value is odd".into(), &["cvalue".into()]);
            }
            accum.len() - orig_len
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
