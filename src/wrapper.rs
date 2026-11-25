use std::{borrow::Borrow, ops::Deref};

use crate::{accum::Levels, Diagnostics, Validate};

/// Wrapper type containing a value which must have been validated.
#[derive(Debug)]
pub struct Valid<T>(T);

// Manually implemented to save depending on the derive feature of serde
#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for Valid<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // don't need to validate here: it's in the wrapper, it's already valid
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::de::Deserialize<'de> + Validate> serde::de::Deserialize<'de> for Valid<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Self::with_levels(value, Levels::FAST).map_err(serde::de::Error::custom)
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
    pub fn with_levels(inner: T, levels: Levels) -> Result<Self, crate::DiagnosticError> {
        inner.validate_levels(levels)?;
        Ok(Self(inner))
    }

    /// Validate the inner value and return the wrapped form.
    pub fn try_new(inner: T) -> Result<Self, crate::DiagnosticError> {
        inner.validate_levels(Default::default())?;
        Ok(Self(inner))
    }
}

/// Factory which produces [Valid]-wrapped instances with consistent validation rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Validator {
    levels: Levels,
}

impl From<Levels> for Validator {
    fn from(levels: Levels) -> Self {
        Self { levels }
    }
}

impl Validator {
    pub fn new(levels: Levels) -> Self {
        Self { levels }
    }

    pub fn validate<T: Validate>(
        &self,
        value: T,
    ) -> Result<(Valid<T>, Diagnostics), crate::DiagnosticError> {
        let diag = value.validate_levels(self.levels)?;
        Ok((Valid(value), diag))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Level, Valid, Validate};

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct MyStruct {
        is_valid: bool,
    }

    impl Validate for MyStruct {
        fn validate_inner(
            &self,
            accum: &mut crate::Accumulator,
        ) -> Result<(), crate::DiagnosticError> {
            if !self.is_valid {
                accum.diagnose_at("is_valid", Level::Error, "struct marked invalid")?;
            }
            Ok(())
        }
    }

    #[test]
    fn test_valid() {
        assert!(Valid::try_new(MyStruct { is_valid: true }).is_ok())
    }

    #[test]
    fn test_invalid() {
        assert!(Valid::try_new(MyStruct { is_valid: false }).is_err())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_ser() {
        let s =
            serde_json::to_string(&Valid::try_new(MyStruct { is_valid: true }).unwrap()).unwrap();
        assert_eq!(s, r#"{"is_valid":true}"#);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_de_valid() {
        let s = r#"{"is_valid":true}"#;
        let _valid: Valid<MyStruct> = serde_json::from_str(s).unwrap();
    }

    #[cfg(feature = "serde")]
    #[test]
    #[should_panic]
    fn test_serde_de_invalid() {
        let s = r#"{"is_valid":false}"#;
        let _valid: Valid<MyStruct> = serde_json::from_str(s).unwrap();
    }
}
