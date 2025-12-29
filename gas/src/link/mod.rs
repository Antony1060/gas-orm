use crate::error::GasError;
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};

mod portable_field_meta;
mod portable_pg_type;

pub use portable_field_meta::*;
pub use portable_pg_type::*;

#[derive(Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct FixedStr<const SIZE: usize = 64>([u8; SIZE]);

impl<const SIZE: usize> TryFrom<&str> for FixedStr<SIZE> {
    type Error = GasError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() >= SIZE {
            return Err(GasError::InternalError(
                format!("string is larger than {} bytes", SIZE - 1).into(),
            ));
        }

        Ok(unsafe { Self::from_panicking(value) })
    }
}

impl<const SIZE: usize> FixedStr<SIZE> {
    #[allow(clippy::missing_safety_doc)]
    pub const unsafe fn from_panicking(value: &str) -> Self {
        if value.len() >= SIZE {
            panic!("value is too long");
        }

        let mut buffer = [0u8; SIZE];
        unsafe {
            std::ptr::copy_nonoverlapping(
                value.as_bytes().as_ptr(),
                buffer.as_mut_ptr(),
                // TODO: buffer overflow
                value.len(),
            );
        }

        Self(buffer)
    }
}

impl From<FixedStr> for String {
    fn from(value: FixedStr) -> Self {
        String::from_utf8_lossy(&value.0).to_string()
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
