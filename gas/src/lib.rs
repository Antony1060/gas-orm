#![allow(private_bounds)]
#![allow(private_interfaces)]

pub mod condition;
pub mod connection;
pub mod eq;
pub mod error;
pub mod field;
pub mod group;
pub mod internals;
pub mod model;
mod ops;
pub mod row;
pub mod sort;
pub mod types;

pub use field::*;
pub use gas_macros::*;
pub use model::*;

pub type GasResult<T> = Result<T, error::GasError>;
