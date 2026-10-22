use tauri::AppHandle;

use super::{app_dirs::settings_store, background::run_blocking};
use crate::{
    home_location,
    models::{FileRecord, MetadataResult, PlaceSuggestion},
};

/// Suggests trip and outing folders. Geocoding runs on the blocking pool.
#[tauri::command]
pub async fn detect_place_suggestions(
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    app: AppHandle,
) -> Result<Vec<PlaceSuggestion>, String> {
    let store = settings_store(&app)?;
    run_blocking(move || home_location::detect_place_suggestions(&store, &files, &metadata_results))
        .await
}
