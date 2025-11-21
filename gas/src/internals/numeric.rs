use crate::internals::AsPgType;
use crate::NaiveDecodable;
use rust_decimal::Decimal;

pub(crate) trait Numeric: AsPgType + NaiveDecodable {}

impl Numeric for i16 {}
impl Numeric for i32 {}
impl Numeric for i64 {}
impl Numeric for f32 {}
impl Numeric for f64 {}
impl Numeric for Decimal {}
