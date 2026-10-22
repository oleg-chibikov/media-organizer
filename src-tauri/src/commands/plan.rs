use std::path::PathBuf;

use tauri::AppHandle;

use super::{
    app_dirs::settings_store,
    background::{run_blocking, spawn_job},
};
use crate::{
    models::{FileRecord, MetadataResult, MoveConfirmationSummary, MovePlan, PlaceSuggestion},
    mover, path_safety, planner,
};

#[tauri::command]
pub async fn create_move_plan(
    selected_path: String,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    place_suggestions: Vec<PlaceSuggestion>,
    app: AppHandle,
) -> Result<(), String> {
    let settings = settings_store(&app)?;
    let layout = run_blocking(move || {
        let selected_folder = PathBuf::from(selected_path);
        path_safety::validate_selected_folder(&selected_folder)?;
        let effective_root = settings.effective_destination_root(&selected_folder)?;
        Ok(planner::DestinationLayout::for_root(&effective_root))
    })
    .await?;
    let place_folders = planner::PlaceFolders::new(&layout, &place_suggestions, &metadata_results)?;

    spawn_job(move || {
        planner::stream_move_plan(&app, &layout, &files, &metadata_results, &place_folders);
    });
    Ok(())
}

#[tauri::command]
pub fn get_move_confirmation_summary(
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    plans: Vec<MovePlan>,
) -> MoveConfirmationSummary {
    mover::build_move_confirmation_summary(&files, &metadata_results, &plans)
}
