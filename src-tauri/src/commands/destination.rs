use tauri::AppHandle;

use super::{app_dirs::settings_store, background::run_blocking, folder_dialog::pick_folder};
use crate::{models::DestinationSettings, settings};

#[tauri::command]
pub async fn get_destination_settings(app: AppHandle) -> Result<DestinationSettings, String> {
    let store = settings_store(&app)?;
    run_blocking(move || store.load()).await
}

#[tauri::command]
pub async fn choose_root_media_folder(
    app: AppHandle,
) -> Result<Option<DestinationSettings>, String> {
    let Some(root_path) = pick_folder(&app).await else {
        return Ok(None);
    };
    let store = settings_store(&app)?;
    run_blocking(move || {
        settings::validate_custom_root_media_folder(root_path.as_ref())?;
        store
            .update(|current| settings::set_root_media_folder(current, Some(root_path)))
            .map(Some)
    })
    .await
}

#[tauri::command]
pub async fn clear_root_media_folder(app: AppHandle) -> Result<DestinationSettings, String> {
    let store = settings_store(&app)?;
    run_blocking(move || store.update(settings::clear_root_media_folder)).await
}

#[tauri::command]
pub async fn set_force_processed_folder(
    force_processed_folder: bool,
    app: AppHandle,
) -> Result<DestinationSettings, String> {
    let store = settings_store(&app)?;
    run_blocking(move || {
        store
            .update(|current| settings::set_force_processed_folder(current, force_processed_folder))
    })
    .await
}
