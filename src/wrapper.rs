use std::{borrow::Borrow, ops::Deref};

use crate::Validate;

/// Wrapper type containing a value which must have been validated.
#[derive(Debug)]
pub struct Valid<T>(T);

#[cfg(feature="serde")]
impl<T: serde::Serialize + Validate> serde::Serialize for Valid<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        self.0.serialize(serializer)
    }
}

#[cfg(feature="serde")]
impl<'de, T: serde::de::Deserialize<'de> + Validate> serde::de::Deserialize<'de> for Valid<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

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
        &self.0
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
