use std::{borrow::Borrow, ops::Deref};

use crate::Validate;

/// Wrapper type containing a value which must have been validated.
#[derive(Debug)]
pub struct Valid<T>(T);

impl<T> Valid<T> {
    /// Borrow a reference to the contained valid value.
    pub fn inner(&self) -> &T {
        &self.0
    }

    /// Unwrap into the contained value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Valid<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<T> AsRef<T> for Valid<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Borrow<T> for Valid<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T: Validate> Valid<T> {
    /// Validate the inner value and return the wrapped form.
    pub fn try_new(inner: T) -> crate::Result<Self> {
        inner.validate()?;
        Ok(Self(inner))
    }
}
