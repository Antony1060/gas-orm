use crate::internals::{AsPgType, NaiveDecodable};
use crate::types::Decimal;

pub trait Numeric: AsPgType + NaiveDecodable {
    type SumType: AsPgType + NaiveDecodable;
}

impl Numeric for i16 {
    type SumType = i64;
}
impl Numeric for i32 {
    type SumType = i64;
}
impl Numeric for i64 {
    type SumType = Decimal;
}
impl Numeric for f32 {
    type SumType = f64;
}
impl Numeric for f64 {
    type SumType = f64;
}
impl Numeric for Decimal {
    type SumType = Decimal;
}
