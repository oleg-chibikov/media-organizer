use std::path::PathBuf;

use tauri::AppHandle;

use crate::{
    models::{FileRecord, MetadataResult, MoveConfirmationSummary, MovePlan},
    mover, path_safety, planner, settings,
};

#[tauri::command]
pub fn create_move_plan(
    selected_path: String,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    app: AppHandle,
) -> Result<(), String> {
    let selected_folder = PathBuf::from(&selected_path);
    path_safety::validate_selected_folder(&selected_folder)?;
    let effective_root =
        settings::resolve_effective_destination_root_for_app(&app, &selected_folder)?;
    let destination_root = PathBuf::from(effective_root.root_path);

    tauri::async_runtime::spawn(async move {
        let _ = planner::stream_move_plan_with_destination_root(
            &app,
            destination_root,
            files,
            metadata_results,
        );
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
