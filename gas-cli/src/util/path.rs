use lazy_static::lazy_static;
use std::path::{Path, PathBuf};

lazy_static! {
    static ref CURRENT_DIR: PathBuf = PathBuf::from(".");
}

pub fn canonicalize_relative_pwd(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let current_dir = std::env::current_dir()?;

    Ok(CURRENT_DIR.join(
        path.as_ref()
            .canonicalize()?
            .strip_prefix(current_dir)
            .map_err(std::io::Error::other)?,
    ))
}
