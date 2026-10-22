use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use super::{
    app_dirs::{history_store, placement_store, settings_store},
    background::{run_blocking, spawn_job},
    session::scan_session,
};
use crate::{
    app_state::AppState,
    models::MovePlan,
    move_run::{self, MoveBatch},
};

#[tauri::command]
pub async fn execute_move_plan(
    selected_path: String,
    scan_id: u64,
    plans: Vec<MovePlan>,
    app: AppHandle,
) -> Result<(), String> {
    let session = scan_session(&app, scan_id)?;
    let file_ids: Vec<String> = plans.iter().map(|plan| plan.file_id.clone()).collect();
    let files = session.records(&file_ids)?;
    let metadata_results = session.metadata(&file_ids);
    let selected_folder = PathBuf::from(selected_path);
    let state = app.state::<AppState>();
    state.picked_folders.ensure_picked(&selected_folder)?;
    let pause = state.move_pause.begin();
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

    spawn_job(move || run.execute(&app, &pause));
    Ok(())
}

/// Stops the running move after the file moving now. Unreached files keep no result.
#[tauri::command]
pub fn pause_move_run(state: State<'_, AppState>) {
    state.move_pause.pause();
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
