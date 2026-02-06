use std::ffi::CStr;
use std::fmt::{Debug, Formatter};

mod portable_field_meta;
mod portable_pg_type;

use crate::error::GasSharedError;
pub use portable_field_meta::*;
pub use portable_pg_type::*;

#[derive(Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct FixedStr<const SIZE: usize = 64>([u8; SIZE]);

impl<const SIZE: usize> TryFrom<&str> for FixedStr<SIZE> {
    type Error = GasSharedError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() >= SIZE {
            return Err(GasSharedError::InternalError(
                format!("string is larger than {} bytes", SIZE - 1).into(),
            ));
        }

        Ok(Self::from_panicking(value))
    }
}

impl<const SIZE: usize> FixedStr<SIZE> {
    pub const fn from_panicking(value: &str) -> Self {
        let bytes = value.as_bytes();
        assert!(bytes.len() <= SIZE);

        let mut buffer = [0u8; SIZE];
        unsafe {
            // SAFETY:
            //  bytes is valid for its length
            //  buffer is valid for SIZE (bytes fit inside buffer, check above)
            //  bytes will not overlap with buffer
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer.as_mut_ptr(), bytes.len());
        }

        Self(buffer)
    }
}

impl<const SIZE: usize> AsRef<str> for FixedStr<SIZE> {
    fn as_ref(&self) -> &str {
        let size = self.0.iter().position(|it| *it == 0).unwrap_or(SIZE);

        unsafe { str::from_utf8_unchecked(&self.0[0..size]) }
    }
}

impl<const SIZE: usize> From<&FixedStr<SIZE>> for String {
    fn from(value: &FixedStr<SIZE>) -> Self {
        value.as_ref().to_string()
    }
}

impl<const SIZE: usize> Debug for FixedStr<SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = CStr::from_bytes_until_nul(&self.0);
        let Ok(str) = str else {
            return write!(f, "<invalid_string>");
        };

        write!(f, "[{str:?}; {}]", SIZE)
    }
}

#[cfg(feature = "serde")]
impl<const SIZE: usize> serde::Serialize for FixedStr<SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

#[cfg(feature = "serde")]
impl<'de, const SIZE: usize> serde::Deserialize<'de> for FixedStr<SIZE> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        Self::try_from(<&str>::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(e.to_string()))
    }
}
