# validatrix

A lightweight validator library for rust.

Validatrix contains no built-in validators, just traits and error types for your own custom validation.

Designed for cases where:

- validateable types are built up of other validateable types
- there is additional schema-level validation
- data is modelled as JSON-like, where sequences are ordered and maps' keys are stringy

## To do

- use smallvec to decrease allocations
- investigate whether RAII can be used instead of pushing and popping accumulator prefixes
