pub type GasCliResult<T> = Result<T, GasCliError>;

#[derive(thiserror::Error, Debug)]
pub enum GasCliError {}
