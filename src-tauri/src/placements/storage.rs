use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::PlacedSuggestion;
use crate::safe_fs::replace_app_file;

const PLACEMENTS_DIR_NAME: &str = "placements";
const PLACEMENTS_FILE_NAME: &str = "placed-suggestions.json";
const PLACEMENTS_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct PlacementsFile {
    version: u32,
    #[serde(default)]
    suggestions: Vec<PlacedSuggestion>,
}

fn placements_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir
        .join(PLACEMENTS_DIR_NAME)
        .join(PLACEMENTS_FILE_NAME)
}

/// A missing or other-version file loads as an empty registry.
pub(super) fn load_placements(app_data_dir: &Path) -> Result<Vec<PlacedSuggestion>, String> {
    let path = placements_path(app_data_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
    let file: PlacementsFile = serde_json::from_str(&raw)
        .map_err(|error| format!("Failed to parse {}: {error}", path.display()))?;
    if file.version != PLACEMENTS_VERSION {
        return Ok(Vec::new());
    }
    Ok(file.suggestions)
}

pub(super) fn save_placements(
    app_data_dir: &Path,
    suggestions: Vec<PlacedSuggestion>,
) -> Result<(), String> {
    let path = placements_path(app_data_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }
    let file = PlacementsFile {
        version: PLACEMENTS_VERSION,
        suggestions,
    };
    let serialized = serde_json::to_string_pretty(&file)
        .map_err(|error| format!("Failed to serialize placed suggestions: {error}"))?;
    replace_app_file(&path, serialized.as_bytes())
        .map_err(|error| format!("Failed to write {}: {error}", path.display()))
}
