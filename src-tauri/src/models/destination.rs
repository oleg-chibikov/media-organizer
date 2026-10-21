use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DestinationRootMode {
    CustomRoot,
    ProcessedFolder,
    /// Files keep their folder and only get the new name.
    InPlace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DestinationSettings {
    pub version: u32,
    pub root_media_folder: Option<String>,
    pub force_processed_folder: bool,
    #[serde(default)]
    pub rename_in_place: bool,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveDestinationRoot {
    pub mode: DestinationRootMode,
    pub root_path: String,
    pub custom_root_configured: bool,
    pub force_processed_folder: bool,
    pub warning: Option<String>,
}
