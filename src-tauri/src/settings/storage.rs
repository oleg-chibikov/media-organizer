use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;

use super::DESTINATION_SETTINGS_VERSION;
use crate::{atomic_file::write_atomically, models::DestinationSettings};

const SETTINGS_DIR_NAME: &str = "settings";
const SETTINGS_FILE_NAME: &str = "destination-settings.json";

pub(super) fn destination_settings_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir
        .join(SETTINGS_DIR_NAME)
        .join(SETTINGS_FILE_NAME)
}

pub(super) fn default_destination_settings() -> DestinationSettings {
    DestinationSettings {
        version: DESTINATION_SETTINGS_VERSION,
        root_media_folder: None,
        force_processed_folder: false,
        updated_at: Some(Utc::now().to_rfc3339()),
    }
}

/// Missing or other-version settings files load as defaults.
pub(super) fn load_destination_settings(
    app_data_dir: &Path,
) -> Result<DestinationSettings, String> {
    let settings_path = destination_settings_path(app_data_dir);
    if !settings_path.exists() {
        return Ok(default_destination_settings());
    }

    let raw = fs::read_to_string(&settings_path).map_err(|error| {
        format!(
            "Failed to read destination settings file at {}: {error}",
            settings_path.display()
        )
    })?;

    let settings: DestinationSettings = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "Failed to parse destination settings file at {}: {error}",
            settings_path.display()
        )
    })?;

    if settings.version != DESTINATION_SETTINGS_VERSION {
        return Ok(default_destination_settings());
    }

    Ok(settings)
}

pub(super) fn save_destination_settings(
    app_data_dir: &Path,
    settings: &DestinationSettings,
) -> Result<DestinationSettings, String> {
    let settings_path = destination_settings_path(app_data_dir);
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Failed to create destination settings directory at {}: {error}",
                parent.display()
            )
        })?;
    }

    let mut persisted = settings.clone();
    persisted.version = DESTINATION_SETTINGS_VERSION;
    persisted.updated_at = Some(Utc::now().to_rfc3339());

    let serialized = serde_json::to_string_pretty(&persisted)
        .map_err(|error| format!("Failed to serialize destination settings JSON: {error}"))?;

    write_atomically(&settings_path, serialized.as_bytes()).map_err(|error| {
        format!(
            "Failed to write destination settings file at {}: {error}",
            settings_path.display()
        )
    })?;

    Ok(persisted)
}
