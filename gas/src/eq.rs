use crate::condition::{Condition, EqExpression};
use crate::pg_param::PgParam;
use crate::types::Decimal;
use crate::Field;

pub trait PgEq<T> {
    fn eq(&self, other: T) -> EqExpression;

    fn neq(&self, other: T) -> EqExpression;

    fn lt(&self, other: T) -> EqExpression;

    fn lte(&self, other: T) -> EqExpression;

    fn gt(&self, other: T) -> EqExpression;

    fn gte(&self, other: T) -> EqExpression;

    fn one_of(&self, other: &[T]) -> EqExpression;
}

pub trait PgEqNone {
    fn is_null(&self) -> EqExpression;

    fn is_not_null(&self) -> EqExpression;
}

impl<T> PgEqNone for Field<Option<T>> {
    fn is_null(&self) -> EqExpression {
        EqExpression::new(Condition::Basic(format!("{} IS NULL", self.name)), vec![])
    }

    fn is_not_null(&self) -> EqExpression {
        EqExpression::new(
            Condition::Basic(format!("{} IS NOT NULL", self.name)),
            vec![],
        )
    }
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
                    Condition::Basic(format!("{}=?", self.full_name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn neq(&self, other: $cmp_type) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!("{}!=?", self.full_name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn lt(&self, other: $cmp_type) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!("{}<?", self.full_name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn lte(&self, other: $cmp_type) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!("{}<=?", self.full_name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn gt(&self, other: $cmp_type) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!("{}>?", self.full_name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn gte(&self, other: $cmp_type) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!("{}>=?", self.full_name)),
                    vec![$pg_param(other.into())],
                )
            }

            fn one_of(&self, other: &[$cmp_type]) -> EqExpression {
                let condition = make_in_expression(self.full_name, other.len());

                EqExpression::new(
                    Condition::Basic(condition),
                    other.iter().map(|it| (*it).into()).map($pg_param).collect(),
                )
            }
        }
    };
}

// text
pg_eq_impl!(String as &str, PgParam::TEXT);
pg_eq_impl!(Option<String> as &str, PgParam::TEXT);

// smallint
pg_eq_impl!(i16 as i8, PgParam::SMALLINT);
pg_eq_impl!(i16 as i16, PgParam::SMALLINT);
pg_eq_impl!(i16 as u8, PgParam::SMALLINT);
pg_eq_impl!(Option<i16> as i8, PgParam::SMALLINT);
pg_eq_impl!(Option<i16> as i16, PgParam::SMALLINT);
pg_eq_impl!(Option<i16> as u8, PgParam::SMALLINT);

// int
pg_eq_impl!(i32 as i8, PgParam::INTEGER);
pg_eq_impl!(i32 as i16, PgParam::INTEGER);
pg_eq_impl!(i32 as i32, PgParam::INTEGER);
pg_eq_impl!(i32 as u8, PgParam::INTEGER);
pg_eq_impl!(i32 as u16, PgParam::INTEGER);
pg_eq_impl!(Option<i32> as i8, PgParam::INTEGER);
pg_eq_impl!(Option<i32> as i16, PgParam::INTEGER);
pg_eq_impl!(Option<i32> as i32, PgParam::INTEGER);
pg_eq_impl!(Option<i32> as u8, PgParam::INTEGER);
pg_eq_impl!(Option<i32> as u16, PgParam::INTEGER);

// bigint
pg_eq_impl!(i64 as i8, PgParam::BIGINT);
pg_eq_impl!(i64 as i16, PgParam::BIGINT);
pg_eq_impl!(i64 as i32, PgParam::BIGINT);
pg_eq_impl!(i64 as i64, PgParam::BIGINT);
pg_eq_impl!(i64 as u8, PgParam::BIGINT);
pg_eq_impl!(i64 as u16, PgParam::BIGINT);
pg_eq_impl!(i64 as u32, PgParam::BIGINT);
pg_eq_impl!(Option<i64> as i8, PgParam::BIGINT);
pg_eq_impl!(Option<i64> as i16, PgParam::BIGINT);
pg_eq_impl!(Option<i64> as i32, PgParam::BIGINT);
pg_eq_impl!(Option<i64> as i64, PgParam::BIGINT);
pg_eq_impl!(Option<i64> as u8, PgParam::BIGINT);
pg_eq_impl!(Option<i64> as u16, PgParam::BIGINT);
pg_eq_impl!(Option<i64> as u32, PgParam::BIGINT);

// real
pg_eq_impl!(f32 as f32, PgParam::REAL);
pg_eq_impl!(f32 as i16, PgParam::REAL);
pg_eq_impl!(f32 as i8, PgParam::REAL);
pg_eq_impl!(f32 as u16, PgParam::REAL);
pg_eq_impl!(f32 as u8, PgParam::REAL);
pg_eq_impl!(Option<f32> as f32, PgParam::REAL);
pg_eq_impl!(Option<f32> as i16, PgParam::REAL);
pg_eq_impl!(Option<f32> as i8, PgParam::REAL);
pg_eq_impl!(Option<f32> as u16, PgParam::REAL);
pg_eq_impl!(Option<f32> as u8, PgParam::REAL);

// double
pg_eq_impl!(f64 as f64, PgParam::DOUBLE);
pg_eq_impl!(f64 as f32, PgParam::DOUBLE);
pg_eq_impl!(f64 as i8, PgParam::DOUBLE);
pg_eq_impl!(f64 as i16, PgParam::DOUBLE);
pg_eq_impl!(f64 as i32, PgParam::DOUBLE);
pg_eq_impl!(f64 as u8, PgParam::DOUBLE);
pg_eq_impl!(f64 as u16, PgParam::DOUBLE);
pg_eq_impl!(f64 as u32, PgParam::DOUBLE);
pg_eq_impl!(Option<f64> as f64, PgParam::DOUBLE);
pg_eq_impl!(Option<f64> as f32, PgParam::DOUBLE);
pg_eq_impl!(Option<f64> as i8, PgParam::DOUBLE);
pg_eq_impl!(Option<f64> as i16, PgParam::DOUBLE);
pg_eq_impl!(Option<f64> as i32, PgParam::DOUBLE);
pg_eq_impl!(Option<f64> as u8, PgParam::DOUBLE);
pg_eq_impl!(Option<f64> as u16, PgParam::DOUBLE);
pg_eq_impl!(Option<f64> as u32, PgParam::DOUBLE);

// decimal
pg_eq_impl!(Decimal as Decimal, PgParam::DECIMAL);
pg_eq_impl!(Decimal as i8, PgParam::DECIMAL);
pg_eq_impl!(Decimal as i16, PgParam::DECIMAL);
pg_eq_impl!(Decimal as i32, PgParam::DECIMAL);
pg_eq_impl!(Decimal as i64, PgParam::DECIMAL);
pg_eq_impl!(Decimal as i128, PgParam::DECIMAL);
pg_eq_impl!(Decimal as isize, PgParam::DECIMAL);
pg_eq_impl!(Decimal as u8, PgParam::DECIMAL);
pg_eq_impl!(Decimal as u16, PgParam::DECIMAL);
pg_eq_impl!(Decimal as u32, PgParam::DECIMAL);
pg_eq_impl!(Decimal as u64, PgParam::DECIMAL);
pg_eq_impl!(Decimal as u128, PgParam::DECIMAL);
pg_eq_impl!(Decimal as usize, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as Decimal, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as i8, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as i16, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as i32, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as i64, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as i128, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as isize, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as u8, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as u16, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as u32, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as u64, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as u128, PgParam::DECIMAL);
pg_eq_impl!(Option<Decimal> as usize, PgParam::DECIMAL);
