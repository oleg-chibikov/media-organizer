use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::{history::HistoryStore, placements::PlacementStore, settings::SettingsStore};

pub(super) fn history_store(app: &AppHandle) -> Result<HistoryStore, String> {
    app_data_dir(app).map(|dir| HistoryStore::in_app_data(&dir))
}

pub(super) fn placement_store(app: &AppHandle) -> Result<PlacementStore, String> {
    app_data_dir(app).map(|dir| PlacementStore::in_app_data(&dir))
}

pub(super) fn settings_store(app: &AppHandle) -> Result<SettingsStore, String> {
    app_data_dir(app).map(|dir| SettingsStore::in_app_data(&dir))
}

/// Where the bundled ExifTool sidecar lives in an installed app.
pub(super) fn resource_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path().resource_dir().ok()
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve appData directory: {error}"))
}
