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
use validatrix::{Validate, Accumulator, Valid};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct A {
    /// Must not be divisible by 3.
    avalue: u8,
    /// Must be valid.
    b: B,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct B {
    /// Must not be divisible by 5.
    bvalue: u8,
    /// All must be valid.
    cs: Vec<C>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct C {
    /// Must not be divisible by 3 and 5.
    cvalue: u8,
}

// Implement `validatrix::Validate` on your structs.
impl Validate for A {
    // `Accumulator` allows you to continue looking for validation errors after the first.
    // But you can return early if you prefer.
    fn validate_inner(&self, accum: &mut Accumulator) {
        if self.avalue % 3 == 0 {
            // Each failure is added with a context: the name of the field
            // (or index of a sequence) which failed.
            accum.add_failure_at("avalue", "fizz");
        }

        // Fields implementing validatrix::Validate can have validation errors accumulated too.
        accum.validate_member_at("b", &self.b);
    }
}

impl Validate for B {
    fn validate_inner(&self, accum: &mut Accumulator) {
        if self.bvalue % 5 == 0 {
            // You can also manually do validation within a prefix context
            accum.with_key("bvalue", |a| a.add_failure("buzz"));
        }

        // Helper method for validating a sequence of validatrix::Validate structs
        accum.validate_iter_at("cs", &self.cs);
    }
}


impl Validate for C {
    fn validate_inner(&self, accum: &mut Accumulator) {
        if (self.cvalue % 3 * self.cvalue % 5) == 0 {
            accum.add_failure_at("cvalue", "fizzbuzz")
        }
    }
}

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
Validation failure(s):
   $.avalue: fizz
   $.b.bvalue: buzz
   $.b.cs[0].cvalue: fizzbuzz
   $.b.cs[1].cvalue: fizzbuzz
".trim());

// the `Valid` wrapper type enforces validity
let valid_wrapped = Valid::try_new(valid.clone()).expect("is valid");
assert!(Valid::try_new(invalid.clone()).is_err());
// `Valid` implements AsRef, Borrow, and Deref for the contained type

#[cfg(feature = "serde")]
{
    // You can also deserialize directly into a Valid;
    // validation errors are raised by serde.
    let valid_wrapped_deser: Valid<A> = serde_json::from_str(
        &serde_json::to_string(&valid).unwrap()
    ).unwrap();

    // serialization is handled transparently
    let invalid_str = serde_json::to_string(&invalid).unwrap();
    assert!(serde_json::from_str::<Valid<A>>(&invalid_str).is_err());
}

```

There is also an asynchronous variant in the `validatrix::asynch` module.
See also `validatrix(::asynch)::ValidateContext`,
which allows passing a reference to some external data as context for the validation.

## Why not

- [validator](https://crates.io/crates/validator)
- [validators](https://crates.io/crates/validators)
- [serde_valid](https://crates.io/crates/serde_valid)

Other validation crates have focused on providing validator functions and proc macros to decorate types.
I found those validators are often trivial implement yourself,
the DSLs for decorating fields just look worse than regular rust code,
and composing custom and built-in validators behaved in unclear ways.

JSONSchema-like validators tend not to be good at schema-level validation.

## Development

Make releases using [cargo-release](https://github.com/crate-ci/cargo-release).

Use [prek](https://github.com/j178/prek) to manage pre-commit hooks.

### To do

- use `Cow<str>` (or alternative like [hipstr](https://crates.io/crates/hipstr), [ecow](https://crates.io/crates/ecow) etc.) for `Failure::message`;
  alternatively, use `Box<dyn Error>` (but then people have to write their own validation errors, although `String`s would still work)
- `Accumulator` could have a fail-fast mode
  - could cap the number of errors at a given value, which might be 1
  - methods would return `Result`s (`Err` if fail-fast is `true`, otherwise `Ok`) so they can be `?`'d and propagate
  - this would cause weirdness in the `&mut self` methods which would then need to cede their failures to the returned errors
- `Key::Field` could use a `Cow` so that custom field names can be used e.g. for maps (or a third variant like `Key::MapKey(String)`)
