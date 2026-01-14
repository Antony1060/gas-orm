use crate::binary::BinaryFields;
use crate::diff::DiffScript;
use crate::error::{GasCliError, GasCliResult};
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Handle;

const MANIFEST_FILE_NAME: &str = "manifest.json";
#[allow(dead_code)]
const SCRIPT_SEPARATOR: &str = "-- GAS_ORM(forward_backward_separator)";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ManifestVersion {
    V1_0_0,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GasManifest {
    pub version: ManifestVersion,
    pub state: BinaryFields,
}

#[derive(thiserror::Error, Debug)]
pub enum GasManifestError {
    #[error("migrations manifest already initialized")]
    AlreadyInitialized,

    #[error("migrations manifest not initialized")]
    NotInitialized,
}

pub struct GasManifestController {
    dir: PathBuf,
}

impl GasManifest {
    pub fn new(fields: BinaryFields) -> Self {
        Self {
            version: ManifestVersion::V1_0_0,
            state: fields,
        }
    }
}

impl GasManifestController {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    // NOTE: maybe more logic?
    fn is_present(&self) -> bool {
        self.dir.exists()
    }

    pub async fn init_with(&self, fields: BinaryFields) -> GasCliResult<GasManifest> {
        if self.is_present() {
            return Err(GasManifestError::AlreadyInitialized.into());
        }

        fs::create_dir_all(&self.dir).await?;
        // fill with initial script
        fs::create_dir_all(self.dir.join("scripts")).await?;

        self.save_fields(fields).await
    }

    pub async fn save_fields(&self, fields: BinaryFields) -> GasCliResult<GasManifest> {
        let manifest_path = self.dir.join(MANIFEST_FILE_NAME);

        let manifest = Handle::current()
            .spawn_blocking(move || {
                let file = std::fs::File::create(manifest_path)?;

                let manifest = GasManifest::new(fields.clone());

                serde_json::to_writer_pretty(file, &manifest)?;

                Ok::<GasManifest, GasCliError>(manifest)
            })
            .await
            .expect("join should have worked")?;

        Ok(manifest)
    }

    #[allow(dead_code)]
    pub async fn save_script(&self, script: DiffScript) -> GasCliResult<()> {
        // TODO: prompt for names, etc
        let name = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();

        let mut file = fs::File::create(format!("{name}.sql")).await?;
        file.write_all(script.forward.as_bytes()).await?;
        file.write_u8(b'\n').await?;
        file.write_all(SCRIPT_SEPARATOR.as_bytes()).await?;
        file.write_u8(b'\n').await?;
        file.write_all(script.backward.as_bytes()).await?;

        Ok(())
    }

    pub async fn load(&self) -> GasCliResult<GasManifest> {
        if !self.is_present() {
            return Err(GasManifestError::NotInitialized.into());
        }

        let manifest_path = self.dir.join(MANIFEST_FILE_NAME);

        let content = fs::read_to_string(manifest_path).await?;
        let manifest = serde_json::from_str(&content)?;

        Ok(manifest)
    }
}
