use crate::error::GasCliError;
use cargo_metadata::CargoOpt;
use std::path::{absolute, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tokio::runtime::Handle;

#[derive(Debug)]
pub struct CargoProject {
    path: PathBuf,
    bin_name: String,
    target_dir: PathBuf,
}

impl CargoProject {
    pub async fn from_path(path: PathBuf) -> Result<Self, GasCliError> {
        let toml_path = path.join("Cargo.toml");
        if !toml_path.exists() {
            return Err(GasCliError::CargoTomlNotFound(toml_path));
        }

        // used to figure out if the project is binary etc
        let manifest_crude = cargo_toml::Manifest::from_path(&toml_path)?;

        // gives information on what the target directory is, all of this could be better,
        //  this is wasteful
        // spawns a std::process::Command
        let manifest_detailed = Handle::current()
            .spawn_blocking(move || {
                cargo_metadata::MetadataCommand::new()
                    .manifest_path(&toml_path)
                    .features(CargoOpt::SomeFeatures(vec![]))
                    .no_deps()
                    .exec()
            })
            .await
            .expect("join should have worked")?;

        if manifest_crude.bin.len() != 1 {
            return Err(GasCliError::InvalidCargoProject(
                "expected the project to have exactly 1 binary defined",
            ));
        }

        let bin = manifest_crude.bin.into_iter().next().unwrap();

        let Some(bin_name) = bin.name else {
            return Err(GasCliError::InvalidCargoProject(
                "the binary in the project doesn't have a defined name",
            ));
        };

        Ok(CargoProject {
            path: absolute(path)?,
            target_dir: manifest_detailed.target_directory.into_std_path_buf(),
            bin_name,
        })
    }

    pub async fn build(&self) -> Result<PathBuf, GasCliError> {
        // TODO: /bin/bash enforces unix-like systems that have bash, probably won't work on windows
        let cargo_build_status = Command::new("/bin/bash")
            .arg("-c")
            .arg(format!("cargo build --bin {}", self.bin_name))
            .current_dir(&self.path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;

        if !cargo_build_status.success() {
            return Err(GasCliError::BinaryParseError(
                "cargo build failed with a non-zero exit code",
            ));
        }

        // eh, maybe we can get this from cargo instead of assuming?
        let bin_path = self.target_dir.join("debug").join(&self.bin_name);

        if !bin_path.exists() {
            return Err(GasCliError::BinaryPathNotFound(bin_path));
        }

        Ok(bin_path)
    }
}
