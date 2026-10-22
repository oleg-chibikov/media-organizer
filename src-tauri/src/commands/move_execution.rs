use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use super::{
    app_dirs::{history_store, placement_store, settings_store},
    background::{run_blocking, spawn_job},
};
use crate::{
    app_state::AppState,
    models::{FileRecord, MetadataResult, MovePlan},
    move_run::{self, MoveBatch},
};

#[tauri::command]
pub async fn execute_move_plan(
    selected_path: String,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    plans: Vec<MovePlan>,
    app: AppHandle,
) -> Result<(), String> {
    let selected_folder = PathBuf::from(selected_path);
    app.state::<AppState>()
        .picked_folders
        .ensure_picked(&selected_folder)?;
    let settings = settings_store(&app)?;
    let history = history_store(&app)?;
    let placements = placement_store(&app)?;
    let batch = MoveBatch {
        files,
        metadata_results,
        plans,
    };
    let sink = app.clone();
    let run = run_blocking(move || {
        move_run::start_move_run(
            &sink,
            &settings,
            &history,
            &placements,
            &selected_folder,
            batch,
        )
    })
    .await?;

    spawn_job(move || run.execute(&app));
    Ok(())
}

/// Removes the empty folders a finished run left behind, the picked folder included.
/// Returns whether the picked folder itself was removed.
#[tauri::command]
pub async fn remove_empty_source_folders(
    selected_path: String,
    app: AppHandle,
) -> Result<bool, String> {
    let selected_folder = PathBuf::from(selected_path);
    app.state::<AppState>()
        .picked_folders
        .ensure_picked(&selected_folder)?;
    let settings = settings_store(&app)?;
    run_blocking(move || move_run::remove_empty_source_folders(&settings, &selected_folder)).await
}
