use std::path::PathBuf;

pub struct CargoProject {
    path: PathBuf,
}

impl CargoProject {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        Ok(CargoProject { path })
    }
}
