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

// all of the value fields here are even, and therefore valid
let valid = A {
    avalue: 0,
    b: B {
        bvalue: 0,
        cs: vec![C { cvalue: 0 }, C { cvalue: 0 }],
    },
};
valid.validate().unwrap();

// all of the value fields are odd, and therefore invalid
let invalid = A {
    avalue: 1,
    b: B {
        bvalue: 1,
        cs: vec![C { cvalue: 1 }, C { cvalue: 1 }],
    },
};
let err = invalid.validate().unwrap_err();
let validation_report = format!("{err}");
assert_eq!(validation_report, "
Validation failure(s):
   $.avalue: value is odd
   $.b.bvalue: value is odd
   $.b.cs[0].cvalue: value is odd
   $.b.cs[1].cvalue: value is odd
".trim())

// the `Valid` wrapper type enforces validity,
// through deserialization or `try_new()`
let valid_wrapped: Valid<A> = serde_json::from_str(
    &serde_json::to_string(&valid).unwrap()
).unwrap();

assert!(serde_json::from_str(
    &serde_json::to_string(&invalid).unwrap()
).is_err())
```

There is also an asynchronous variant in the `validatrix::asynch` module.
Additionally, there is `validatrix::Valid`,
a wrapper type which can only be created by validating its inner value.
Finally, there is `validatrix(::asynch)::ValidateContext`,
which allows passing a reference to some external data as context for the validation.

## To do

- ~~use smallvec to decrease allocations~~ _Doesn't really help_
- investigate whether RAII can be used instead of pushing and popping accumulator prefixes
