use crate::condition::{Condition, EqExpression};
use crate::field::Field;
use crate::internals::PgParam;
use crate::types::Decimal;
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};

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

pub trait PgEqTime {
    fn is_now(&self) -> EqExpression;
    fn is_before_now(&self) -> EqExpression;
    fn is_now_or_before(&self) -> EqExpression;
    fn is_after_now(&self) -> EqExpression;
    fn is_now_or_after(&self) -> EqExpression;
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

// timestamp
pg_eq_impl!(NaiveDateTime as NaiveDateTime, PgParam::TIMESTAMP);
pg_eq_impl!(Option<NaiveDateTime> as NaiveDateTime, PgParam::TIMESTAMP);

// timestamp with timezone
pg_eq_impl!(DateTime<Utc> as DateTime<Utc>, PgParam::TIMESTAMP_TZ_UTC);
pg_eq_impl!(DateTime<Utc> as DateTime<Local>, PgParam::TIMESTAMP_TZ_LOCAL);
pg_eq_impl!(DateTime<Utc> as DateTime<FixedOffset>, PgParam::TIMESTAMP_TZ_FIXED_OFFSET);
pg_eq_impl!(DateTime<Local> as DateTime<Utc>, PgParam::TIMESTAMP_TZ_UTC);
pg_eq_impl!(DateTime<Local> as DateTime<Local>, PgParam::TIMESTAMP_TZ_LOCAL);
pg_eq_impl!(DateTime<Local> as DateTime<FixedOffset>, PgParam::TIMESTAMP_TZ_FIXED_OFFSET);
pg_eq_impl!(DateTime<FixedOffset> as DateTime<Utc>, PgParam::TIMESTAMP_TZ_UTC);
pg_eq_impl!(DateTime<FixedOffset> as DateTime<Local>, PgParam::TIMESTAMP_TZ_LOCAL);
pg_eq_impl!(DateTime<FixedOffset> as DateTime<FixedOffset>, PgParam::TIMESTAMP_TZ_FIXED_OFFSET);
pg_eq_impl!(Option<DateTime<Utc>> as DateTime<Utc>, PgParam::TIMESTAMP_TZ_UTC);
pg_eq_impl!(Option<DateTime<Utc>> as DateTime<Local>, PgParam::TIMESTAMP_TZ_LOCAL);
pg_eq_impl!(Option<DateTime<Utc>> as DateTime<FixedOffset>, PgParam::TIMESTAMP_TZ_FIXED_OFFSET);
pg_eq_impl!(Option<DateTime<Local>> as DateTime<Utc>, PgParam::TIMESTAMP_TZ_UTC);
pg_eq_impl!(Option<DateTime<Local>> as DateTime<Local>, PgParam::TIMESTAMP_TZ_LOCAL);
pg_eq_impl!(Option<DateTime<Local>> as DateTime<FixedOffset>, PgParam::TIMESTAMP_TZ_FIXED_OFFSET);
pg_eq_impl!(Option<DateTime<FixedOffset>> as DateTime<Utc>, PgParam::TIMESTAMP_TZ_UTC);
pg_eq_impl!(Option<DateTime<FixedOffset>> as DateTime<Local>, PgParam::TIMESTAMP_TZ_LOCAL);
pg_eq_impl!(Option<DateTime<FixedOffset>> as DateTime<FixedOffset>, PgParam::TIMESTAMP_TZ_FIXED_OFFSET);

// date
pg_eq_impl!(NaiveDate as NaiveDate, PgParam::DATE);
pg_eq_impl!(Option<NaiveDate> as NaiveDate, PgParam::DATE);

// time
pg_eq_impl!(NaiveTime as NaiveTime, PgParam::TIME);
pg_eq_impl!(Option<NaiveTime> as NaiveTime, PgParam::TIME);

macro_rules! pg_timed_now_impl {
    ($field_type:ty, $time_cast:literal) => {
        impl PgEqTime for Field<$field_type> {
            fn is_now(&self) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!(concat!("{}=NOW()::", $time_cast), self.full_name)),
                    vec![],
                )
            }

            fn is_before_now(&self) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!(concat!("{}<NOW()::", $time_cast), self.full_name)),
                    vec![],
                )
            }

            fn is_now_or_before(&self) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!(concat!("{}<=NOW()::", $time_cast), self.full_name)),
                    vec![],
                )
            }

            fn is_after_now(&self) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!(concat!("{}>NOW()::", $time_cast), self.full_name)),
                    vec![],
                )
            }

            fn is_now_or_after(&self) -> EqExpression {
                EqExpression::new(
                    Condition::Basic(format!(concat!("{}>=NOW()::", $time_cast), self.full_name)),
                    vec![],
                )
            }
        }
    };
}

// timestamp
pg_timed_now_impl!(NaiveDateTime, "timestamp");
pg_timed_now_impl!(Option<NaiveDateTime>, "timestamp");

// timestamp with timezone
pg_timed_now_impl!(DateTime<Utc>, "timestamp");
pg_timed_now_impl!(DateTime<Local>, "timestamp");
pg_timed_now_impl!(DateTime<FixedOffset>, "timestamp");
pg_timed_now_impl!(Option<DateTime<Utc>>, "timestamp");
pg_timed_now_impl!(Option<DateTime<Local>>, "timestamp");
pg_timed_now_impl!(Option<DateTime<FixedOffset>>, "timestamp");

// date
pg_timed_now_impl!(NaiveDate, "date");
pg_timed_now_impl!(Option<NaiveDate>, "date");

// time
pg_timed_now_impl!(NaiveTime, "time");
pg_timed_now_impl!(Option<NaiveTime>, "time");
