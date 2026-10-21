use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::{
    models::DestinationSettings,
    settings::{self, load_destination_settings_for_app, save_destination_settings_for_app},
};

#[tauri::command]
pub fn get_destination_settings(app: AppHandle) -> Result<DestinationSettings, String> {
    load_destination_settings_for_app(&app)
}

#[tauri::command]
pub fn choose_root_media_folder(app: AppHandle) -> Result<Option<DestinationSettings>, String> {
    let Some(selected_path) = app.dialog().file().blocking_pick_folder() else {
        return Ok(None);
    };

    let root_path = PathBuf::from(selected_path.to_string());
    settings::validate_custom_root_media_folder(&root_path)?;

    let current = load_destination_settings_for_app(&app)?;
    let updated =
        settings::set_root_media_folder(&current, Some(root_path.to_string_lossy().to_string()));
    save_destination_settings_for_app(&app, &updated).map(Some)
}

#[tauri::command]
pub fn clear_root_media_folder(app: AppHandle) -> Result<DestinationSettings, String> {
    let current = load_destination_settings_for_app(&app)?;
    let updated = settings::clear_root_media_folder(&current);
    save_destination_settings_for_app(&app, &updated)
}

#[tauri::command]
pub fn set_force_processed_folder(
    force_processed_folder: bool,
    app: AppHandle,
) -> Result<DestinationSettings, String> {
    let current = load_destination_settings_for_app(&app)?;
    let updated = settings::set_force_processed_folder(&current, force_processed_folder);
    save_destination_settings_for_app(&app, &updated)
}
