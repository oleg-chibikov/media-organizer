use serde::{Deserialize, Serialize};

use super::HomeLocation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DestinationRootMode {
    CustomRoot,
    ProcessedFolder,
    /// Files keep their folder and only get the new name.
    InPlace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DestinationSettings {
    pub version: u32,
    pub root_media_folder: Option<String>,
    pub force_processed_folder: bool,
    #[serde(default)]
    pub rename_in_place: bool,
    /// Trips and outings go to place folders. Rename in place turns this off.
    #[serde(default = "group_by_place_default")]
    pub group_by_place: bool,
    /// Trips are measured from here. Unset until learned or picked.
    #[serde(default)]
    pub home: Option<HomeLocation>,
    pub updated_at: Option<String>,
}

const fn group_by_place_default() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveDestinationRoot {
    pub mode: DestinationRootMode,
    pub root_path: String,
    pub custom_root_configured: bool,
    pub force_processed_folder: bool,
    pub warning: Option<String>,
}
