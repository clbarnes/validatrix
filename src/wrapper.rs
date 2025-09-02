use std::{borrow::Borrow, ops::Deref};

use crate::{Validate, ValidateContext};

/// Wrapper type containing a value which must have been validated.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
pub struct Valid<T>(T);

#[cfg(feature = "serde")]
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

impl<T: ValidateContext> Valid<T> {
    pub fn try_new_with_context(inner: T, context: &T::Context) -> crate::Result<Self> {
        inner.validate(context)?;
        Ok(Self(inner))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Valid, Validate};

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct MyStruct {
        is_valid: bool,
    }

    impl Validate for MyStruct {
        fn validate_inner(&self, accum: &mut crate::Accumulator) -> usize {
            if self.is_valid {
                0
            } else {
                accum.add_failure("struct marked invalid".into(), &["is_valid".into()]);
                1
            }
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
