use crate::binary::BinaryFields;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ManifestVersion {
    V1_0_0,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GasManifest {
    pub version: ManifestVersion,
    pub state: BinaryFields,
}

impl GasManifest {
    pub fn new(fields: BinaryFields) -> Self {
        Self {
            version: ManifestVersion::V1_0_0,
            state: fields,
        }
    }
}
