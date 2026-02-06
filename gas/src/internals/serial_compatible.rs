use crate::internals::{AsPgType, NaiveDecodable, Numeric};

pub trait SerialCompatible: Numeric + AsPgType + NaiveDecodable {}

impl SerialCompatible for i16 {}

impl SerialCompatible for i32 {}

impl SerialCompatible for i64 {}

pub const fn assert_serial<T: SerialCompatible>() {}
