# validatrix

A lightweight validator library for rust.

Validatrix contains no built-in validators, just traits and error types for your own custom validation.

Designed for cases where:

- possibly-valid types are built up of other possibly-valid types
- there is additional schema-level validation
- data is modelled as JSON-like, where sequences are ordered and maps' keys are stringy

The `Display` implementation of `validatrix::Error` can list multiple validation errors,
pointing to the location of the errors with [JSONPath](https://jsonpath.com/)-like syntax,
although implementors can choose to fail fast instead.

## Usage

The example below has two parts:

- `my_implementation` represents library code defining structures and how to validate them
- `my_consumption` represents dependant code using that validation

Types are re-exported under the `implement` and `consume` modules for these two use cases.

```rust
mod my_implementation {
    use validatrix::implement::{Accumulator, DiagnosticError, Level, Validate};

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct A {
        /// Must not be divisible by 3.
        pub avalue: u8,
        /// Must be valid.
        pub b: B,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct B {
        /// Must not be divisible by 5.
        pub bvalue: u8,
        /// All must be valid.
        pub cs: Vec<C>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct C {
        /// Must not be divisible by 3 and 5.
        pub cvalue: u8,
    }

    // Implement `Validate` on your structs.
    impl Validate for A {
        // `Accumulator` allows you to continue looking for validation errors after the first.
        // Its methods return an error when validation should stop early,
        // so propagate them with `?`.
        fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError> {
            if self.avalue % 3 == 0 {
                // Each diagnostic is added with a context: the name of the field
                // (or index of a sequence) which failed.
                accum.diagnose_at("avalue", Level::Error, "fizz")?;
            }

            // Fields implementing `Validate` can have diagnostics accumulated too.
            accum.validate_member_at("b", &self.b)
        }
    }

    impl Validate for B {
        fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError> {
            if self.bvalue % 5 == 0 {
                // You can also manually do validation within a prefix context
                accum.with_key("bvalue", |a| a.diagnose(Level::Error, "buzz"))?;
            }

            // Helper method for validating a sequence of `Validate` structs
            accum.validate_iter_at("cs", &self.cs)
        }
    }

    impl Validate for C {
        fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError> {
            if (self.cvalue % 3 * self.cvalue % 5) == 0 {
                accum.diagnose_at("cvalue", Level::Error, "fizzbuzz")?;
            }
            Ok(())
        }
    }
}

mod my_consumption {
    use validatrix::consume::{Valid, Validate};

    use super::my_implementation::{A, B, C};

    pub fn run() {
        // valid
        let valid = A {
            avalue: 1,
            b: B {
                bvalue: 1,
                cs: vec![C { cvalue: 1 }, C { cvalue: 1 }],
            },
        };
        valid.validate().unwrap();

        // all of the value fields are fizz/buzz, and therefore invalid
        let invalid = A {
            avalue: 3,
            b: B {
                bvalue: 5,
                cs: vec![C { cvalue: 15 }, C { cvalue: 30 }],
            },
        };
        let err = invalid.validate().unwrap_err();
        let validation_report = format!("{err}");
        assert_eq!(validation_report, "
Validation diagnostics:
   ERROR @ $.avalue: fizz
   ERROR @ $.b.bvalue: buzz
   ERROR @ $.b.cs[0].cvalue: fizzbuzz
   ERROR @ $.b.cs[1].cvalue: fizzbuzz
".trim());

        // the `Valid` wrapper type enforces validity
        let valid_wrapped = Valid::try_new(valid.clone()).expect("is valid");
        assert!(Valid::try_new(invalid.clone()).is_err());
        // `Valid` implements AsRef, Borrow, and Deref for the contained type

        // You can also deserialize directly into a Valid;
        // validation errors are raised by serde.
        let valid_wrapped_deser: Valid<A> = serde_json::from_str(
            &serde_json::to_string(&valid).unwrap()
        ).unwrap();

        // serialization is handled transparently
        let invalid_str = serde_json::to_string(&invalid).unwrap();
        assert!(serde_json::from_str::<Valid<A>>(&invalid_str).is_err());
    }
}

fn main() {
    my_consumption::run();
}
```

Validation behaviour including ignoring low-level diagnostics and early returning on high-level diagnostics can be fine-tuned with the `Validate::validate_policy` method.

## Why not

- [validator](https://crates.io/crates/validator)
- [validators](https://crates.io/crates/validators)
- [serde_valid](https://crates.io/crates/serde_valid)

Other validation crates have focused on providing validator functions and proc macros to decorate types.
In my opinion:

- those validators are often trivial implement yourself
- a broad variety of validators you may not use bring dependencies with them (e.g. [regex](https://crates.io/crates/regex), [email_address](https://crates.io/crates/email_address))
- the DSLs for decorating fields just look worse than regular rust code
- composing custom and built-in validators behaved in unclear ways

Additionally, JSONSchema-like validators tend not to be good at schema-level validation,
where you need rules like "these two `Vec`s must be the same length".

## Development

Make releases using [cargo-release](https://github.com/crate-ci/cargo-release).

Use [prek](https://github.com/j178/prek) to manage pre-commit hooks.
