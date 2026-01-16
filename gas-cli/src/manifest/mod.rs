use crate::binary::BinaryFields;
use crate::error::{GasCliError, GasCliResult};
use crate::sync::MigrationScript;
use crate::util;
use chrono::Utc;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Handle;

const MANIFEST_FILE_NAME: &str = "manifest.json";
const SCRIPT_SEPARATOR: &str = "-- GAS_ORM(forward_backward_separator)";
const SCRIPTS_DIR: &str = "scripts";

lazy_static! {
    static ref NON_ALPHANUMERIC_REGEX: Regex =
        Regex::new(r"[^a-zA-Z0-9_-]").expect("invalid regex");
}

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
        // TODO: fill with initial script
        fs::create_dir_all(self.dir.join(SCRIPTS_DIR)).await?;

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

    pub async fn save_script(
        &self,
        pretty_name: &str,
        script: &MigrationScript,
    ) -> GasCliResult<PathBuf> {
        let script_path = self.process_script_name(pretty_name).await?;

        let mut file = fs::File::create(&script_path).await?;
        file.write_all(script.forward.as_bytes()).await?;
        file.write_all(SCRIPT_SEPARATOR.as_bytes()).await?;
        file.write_u8(b'\n').await?;
        file.write_all(script.backward.as_bytes()).await?;

        Ok(script_path)
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

    // checks for name conflicting files and adds a suffix
    // will return the first path that is available
    async fn process_script_name(&self, name: &str) -> GasCliResult<PathBuf> {
        let time_formatted = Utc::now().format("%Y-%m-%d");

        let scripts_dir = self.dir.join(SCRIPTS_DIR);
        let current_scripts: Vec<_> = {
            let dir_files = util::file::list_dir(&scripts_dir).await?;
            let mut scripts = Vec::with_capacity(dir_files.len());

            for dir_file in dir_files {
                if !dir_file.file_type().await?.is_file() {
                    continue;
                }

                scripts.push(dir_file);
            }

            scripts
        };

        // append sequence number to migrations made the same day
        let script_start = 'seq_loop: {
            let mut seq = 0usize;
            loop {
                let script_start = format!("{}_{:02}", time_formatted, seq);

                seq += 1;

                // linear search here should be fine
                if !current_scripts.iter().any(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(&script_start)
                }) {
                    break 'seq_loop script_start;
                }
            }
        };

        let name = format!(
            "{}_{}.sql",
            script_start,
            NON_ALPHANUMERIC_REGEX.replace_all(name, "_")
        );

        Ok(scripts_dir.join(name))
    }
}
