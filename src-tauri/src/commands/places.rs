use super::background::run_blocking;
use crate::{
    models::{FileRecord, MetadataResult, PlaceSuggestion},
    trips,
};

/// Suggests trip and outing folders. Geocoding runs on the blocking pool.
#[tauri::command]
pub async fn detect_place_suggestions(
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
) -> Result<Vec<PlaceSuggestion>, String> {
    run_blocking(move || {
        Ok(trips::detect_place_suggestions(
            &files,
            &metadata_results,
            None,
        ))
    })
    .await
}
