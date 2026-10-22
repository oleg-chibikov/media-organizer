use std::path::PathBuf;

use tauri::AppHandle;

use super::{
    app_dirs::{placement_store, settings_store},
    background::run_blocking,
};
use crate::{
    home_location,
    models::{FileRecord, MetadataResult, PlaceDetection},
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
) -> Result<PlaceDetection, String> {
    let settings = settings_store(&app)?;
    let placements = placement_store(&app)?;
    run_blocking(move || {
        let mut detection =
            home_location::detect_place_suggestions(&settings, &files, &metadata_results)?;
        let selected_folder = PathBuf::from(selected_path);
        detection.suggestions = placements::continue_placed_suggestions(
            &placements,
            &settings,
            &selected_folder,
            &files,
            &metadata_results,
            detection.suggestions,
        )?;
        Ok(detection)
    })
    .await
}
