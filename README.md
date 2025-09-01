# validatrix

A lightweight validator library for rust.

Validatrix contains no built-in validators, just traits and error types for your own custom validation.

Designed for cases where:

- validateable types are built up of other validateable types
- there is additional schema-level validation
- data is modelled as JSON-like, where sequences are ordered and maps' keys are stringy

## Usage

```rust
use validatrix::{Validate, Accumulator};

struct A {
    avalue: u8,
    b: B,
}

// Implement `validatrix::Validate` on your structs.
impl Validate for A {
    // `Accumulator` allows you to continue looking for validation errors after the first.
    // But you can return early if you prefer.
    fn validate_inner(&self, accum: &mut Accumulator) -> usize {
        let orig = accum.len();
        if self.avalue % 2 != 0 {
            // Each failure is added with a context: the name of the field
            // (or index of a sequence) which failed.
            accum.add_failure("value is odd".into(), &["avalue".into()]);
        }

        // Fields implementing validatrix::Validate can be accumulated too.
        accum.prefix.push("b".into());
        self.b.validate_inner(accum);
        accum.prefix.pop();

        // Return the number of new errors in this call
        accum.len() - orig
    }
}

struct B {
    bvalue: u8,
    cs: Vec<C>,
}

impl Validate for B {
    fn validate_inner(&self, accum: &mut Accumulator) -> usize {
        let orig_len = accum.len();
        if self.bvalue % 2 != 0 {
            accum.add_failure("value is odd".into(), &["bvalue".into()]);
        }

        // Helper method for validating a sequence of validatrix::Validate structs
        accum.validate_iter("cs", &self.cs);

        accum.len() - orig_len
    }
}

struct C {
    cvalue: u8,
}

impl Validate for C {
    fn validate_inner(&self, accum: &mut Accumulator) -> usize {
        let orig_len = accum.len();
        if self.cvalue % 2 != 0 {
            accum.add_failure("value is odd".into(), &["cvalue".into()]);
        }
        accum.len() - orig_len
    }
}

// all of the value fields are odd, and therefore invalid
let valid = A {
    avalue: 1,
    b: B {
        bvalue: 1,
        cs: vec![C { cvalue: 1 }, C { cvalue: 1 }],
    },
};
let err = valid.validate().unwrap_err();
let validation_report = format!("{err}");
assert_eq!(validation_report, "
Validation failure(s):
   $.avalue: value is odd
   $.b.bvalue: value is odd
   $.b.cs[0].cvalue: value is odd
   $.b.cs[1].cvalue: value is odd
".trim())
```

## To do

- ~~use smallvec to decrease allocations~~ _Doesn't really help_
- investigate whether RAII can be used instead of pushing and popping accumulator prefixes
