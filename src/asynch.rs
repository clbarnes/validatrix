use crate::errors::Accumulator;

/// Trait for asynchronous validation.
#[allow(async_fn_in_trait)]
pub trait Validate {
    async fn validate(&self) -> crate::Result {
        let mut accum = Default::default();
        self.validate_inner(&mut accum).await;
        accum.into()
    }

    async fn validate_inner(&self, accum: &mut Accumulator) -> usize;
}

/// Trait for asynchronous validation where some external data or context is required.
#[allow(async_fn_in_trait)]
pub trait ValidateContext {
    type Context;

    async fn validate(&self, context: &Self::Context) -> crate::Result {
        let mut accum = Default::default();
        self.validate_inner(context, &mut accum).await;
        accum.into()
    }

    async fn validate_inner(&self, context: &Self::Context, accum: &mut Accumulator) -> usize;
}
