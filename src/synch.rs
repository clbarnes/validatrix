use crate::errors::Accumulator;

/// Trait for synchronous validation.
pub trait Validate {
    /// Perform top-level validation on this value.
    ///
    /// Should not be called inside other validators;
    /// use [Validate::validate_inner] instead.
    fn validate(&self) -> crate::Result {
        let mut accum = Default::default();
        self.validate_inner(&mut accum);
        accum.into()
    }

    /// Accumulate validation errors.
    ///
    /// Validators of containing types should call this;
    /// end users probably want [Validate::validate] instead.
    fn validate_inner(&self, accum: &mut Accumulator) -> usize;
}

/// Trait for synchronous validation where some external data or context is required.
pub trait ValidateContext {
    /// Type of context which the validator needs (external data, resources etc.)
    type Context;

    /// Perform top-level validation on this value, with the given context.
    ///
    /// Should not be called inside other validators;
    /// use [ValidateContext::validate_inner] instead.
    fn validate(&self, context: &Self::Context) -> crate::Result {
        let mut accum = Default::default();
        self.validate_inner(context, &mut accum);
        accum.into()
    }

    /// Accumulate validation errors.
    ///
    /// Validators of containing types should call this;
    /// end users probably want [ValidateContext::validate] instead.
    fn validate_inner(&self, context: &Self::Context, accum: &mut Accumulator) -> usize;
}
