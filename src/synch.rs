use crate::errors::Accumulator;

/// Trait for validation.
pub trait Validate {
    fn validate(&self) -> crate::Result {
        let mut accum = Default::default();
        self.validate_inner(&mut accum);
        accum.into()
    }

    fn validate_inner(&self, accum: &mut Accumulator) -> usize;
}

/// Trait for validation where some external data or context is required.
pub trait ValidateContext {
    type Context;

    fn validate(&self, context: &Self::Context) -> crate::Result {
        let mut accum = Default::default();
        self.validate_inner(context, &mut accum);
        accum.into()
    }
    
    fn validate_inner(&self, context: &Self::Context, accum: &mut Accumulator) -> usize;
}
