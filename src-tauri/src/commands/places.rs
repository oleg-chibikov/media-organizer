use std::path::PathBuf;

use tauri::AppHandle;

use super::{
    app_dirs::{placement_store, settings_store},
    background::run_blocking,
};
use crate::{
    home_location,
    models::{FileRecord, MetadataResult, PlaceSuggestion},
    placements,
};

/// Suggests trip and outing folders, continuing the ones earlier runs filled under the
/// same destination root. Geocoding runs on the blocking pool.
#[tauri::command]
pub async fn detect_place_suggestions(
    selected_path: String,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    app: AppHandle,
) -> Result<Vec<PlaceSuggestion>, String> {
    let settings = settings_store(&app)?;
    let placements = placement_store(&app)?;
    run_blocking(move || {
        let found = home_location::detect_place_suggestions(&settings, &files, &metadata_results)?;
        let selected_folder = PathBuf::from(selected_path);
        placements::continue_placed_suggestions(&placements, &settings, &selected_folder, found)
    })
    .await
}
