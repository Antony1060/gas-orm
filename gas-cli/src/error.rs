use crate::manifest::GasManifestError;
use std::borrow::Cow;
use std::path::PathBuf;

pub type GasCliResult<T> = Result<T, GasCliError>;

#[derive(thiserror::Error, Debug)]
pub enum GasCliError {
    #[error(transparent)]
    ClapError(#[from] clap::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("system time error: {0}")]
    TimeError(#[from] std::time::SystemTimeError),

    #[error("Cargo.toml not found in project path, expected at: {0}")]
    CargoTomlNotFound(PathBuf),

    #[error("Cargo project found in an unexpected state: {0}")]
    InvalidCargoProject(&'static str),

    #[error("Cargo.toml parsing error: {0}")]
    ManifestParseErrorCargoToml(#[from] cargo_toml::Error),

    #[error("Cargo.toml parsing error: {0}")]
    ManifestParseErrorCargoMetadata(#[from] cargo_metadata::Error),

    #[error("compiled binary not found after build, expected at: {0}")]
    BinaryPathNotFound(PathBuf),

    #[error("failed to parse binary: {0}")]
    BinaryParseError(&'static str),

    #[error("failed to parse binary: {0}")]
    ObjectBinaryParseError(#[from] object::Error),

    #[error("failed to (de)serialize: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("manifest error: {0}")]
    ManifestError(#[from] GasManifestError),

    #[error("interaction error: {0}")]
    DialoguerError(#[from] dialoguer::Error),

    #[error("failed to generate a migration: {reason}")]
    MigrationsGenerationError { reason: Cow<'static, str> },

    #[error("general failure")]
    GeneralFailure,
}
