use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::models::{DestinationSettings, EffectiveDestinationRoot};

pub(crate) fn load_destination_settings_for_app(
    app: &AppHandle,
) -> Result<DestinationSettings, String> {
    super::load_destination_settings(&app_data_dir(app)?)
}

pub(crate) fn save_destination_settings_for_app(
    app: &AppHandle,
    destination_settings: &DestinationSettings,
) -> Result<DestinationSettings, String> {
    super::save_destination_settings(&app_data_dir(app)?, destination_settings)
}

pub(crate) fn resolve_effective_destination_root_for_app(
    app: &AppHandle,
    selected_folder: &Path,
) -> Result<EffectiveDestinationRoot, String> {
    let destination_settings = load_destination_settings_for_app(app)?;
    super::resolve_effective_destination_root(selected_folder, &destination_settings)
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve appData directory: {error}"))
}
