use crate::errors::Accumulator;

pub trait Validate {
    fn validate(&self) -> crate::Result {
        let mut accum = Default::default();
        self.validate_inner(&mut accum);
        accum.into()
    }

    fn validate_inner(&self, accum: &mut Accumulator) -> usize;
}
