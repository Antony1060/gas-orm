use crate::condition::{Condition, EqExpression};
use crate::types::Decimal;
use crate::{Field, PgParams};

pub trait PgEq<T> {
    fn eq(&self, other: T) -> EqExpression;

    fn neq(&self, other: T) -> EqExpression;

    fn lt(&self, other: T) -> EqExpression;

    fn lte(&self, other: T) -> EqExpression;

    fn gt(&self, other: T) -> EqExpression;

    fn gte(&self, other: T) -> EqExpression;

    fn one_of(&self, other: &[T]) -> EqExpression;
}

fn make_in_expression(name: &str, params: usize) -> String {
    format!(
        "{} IN ({})",
        name,
        (0..params)
            .map(|_| '?')
            .fold(String::new(), |acc, curr| format!("{acc}, {curr}"))
    )
}

macro_rules! pg_eq_impl {
    ($field_type:ty as $cmp_type:ty, $pg_param:expr) => {
        impl PgEq<$cmp_type> for Field<$field_type> {
            fn eq(&self, other: $cmp_type) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!("{}=?", self.name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn neq(&self, other: $cmp_type) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!("{}!=?", self.name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn lt(&self, other: $cmp_type) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!("{}<?", self.name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn lte(&self, other: $cmp_type) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!("{}<=?", self.name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn gt(&self, other: $cmp_type) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!("{}>?", self.name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn gte(&self, other: $cmp_type) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!("{}>=?", self.name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn one_of(&self, other: &[$cmp_type]) -> EqExpression {
                let condition = make_in_expression(self.name, other.len());

                EqExpression::new(
                    Condition::Basic(condition),
                    other.iter().map(|it| (*it).into()).map($pg_param).collect(),
                )
            }
        }
    };
}

// text
pg_eq_impl!(String as &str, PgParams::TEXT);

// smallint
pg_eq_impl!(i16 as i8, PgParams::SMALLINT);
pg_eq_impl!(i16 as i16, PgParams::SMALLINT);
pg_eq_impl!(i16 as u8, PgParams::SMALLINT);

// int
pg_eq_impl!(i32 as i8, PgParams::INTEGER);
pg_eq_impl!(i32 as i16, PgParams::INTEGER);
pg_eq_impl!(i32 as i32, PgParams::INTEGER);
pg_eq_impl!(i32 as u8, PgParams::INTEGER);
pg_eq_impl!(i32 as u16, PgParams::INTEGER);

// bigint
pg_eq_impl!(i64 as i8, PgParams::BIGINT);
pg_eq_impl!(i64 as i16, PgParams::BIGINT);
pg_eq_impl!(i64 as i32, PgParams::BIGINT);
pg_eq_impl!(i64 as i64, PgParams::BIGINT);
pg_eq_impl!(i64 as u8, PgParams::BIGINT);
pg_eq_impl!(i64 as u16, PgParams::BIGINT);
pg_eq_impl!(i64 as u32, PgParams::BIGINT);

// real
pg_eq_impl!(f32 as f32, PgParams::REAL);
pg_eq_impl!(f32 as i16, PgParams::REAL);
pg_eq_impl!(f32 as i8, PgParams::REAL);
pg_eq_impl!(f32 as u16, PgParams::REAL);
pg_eq_impl!(f32 as u8, PgParams::REAL);

// double
pg_eq_impl!(f64 as f64, PgParams::DOUBLE);
pg_eq_impl!(f64 as f32, PgParams::DOUBLE);
pg_eq_impl!(f64 as i8, PgParams::DOUBLE);
pg_eq_impl!(f64 as i16, PgParams::DOUBLE);
pg_eq_impl!(f64 as i32, PgParams::DOUBLE);
pg_eq_impl!(f64 as u8, PgParams::DOUBLE);
pg_eq_impl!(f64 as u16, PgParams::DOUBLE);
pg_eq_impl!(f64 as u32, PgParams::DOUBLE);

// decimal
pg_eq_impl!(Decimal as Decimal, PgParams::DECIMAL);
pg_eq_impl!(Decimal as i8, PgParams::DECIMAL);
pg_eq_impl!(Decimal as i16, PgParams::DECIMAL);
pg_eq_impl!(Decimal as i32, PgParams::DECIMAL);
pg_eq_impl!(Decimal as i64, PgParams::DECIMAL);
pg_eq_impl!(Decimal as i128, PgParams::DECIMAL);
pg_eq_impl!(Decimal as isize, PgParams::DECIMAL);
pg_eq_impl!(Decimal as u8, PgParams::DECIMAL);
pg_eq_impl!(Decimal as u16, PgParams::DECIMAL);
pg_eq_impl!(Decimal as u32, PgParams::DECIMAL);
pg_eq_impl!(Decimal as u64, PgParams::DECIMAL);
pg_eq_impl!(Decimal as u128, PgParams::DECIMAL);
pg_eq_impl!(Decimal as usize, PgParams::DECIMAL);
