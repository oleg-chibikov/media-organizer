use std::path::PathBuf;

use tauri::AppHandle;

use crate::{
    events, history,
    models::{FileRecord, HistoryIndexUpdatedEvent, MetadataResult, MovePlan, MoveProgressPhase},
    move_run::{self, progress::emit_move_progress_message, MoveRunRequest},
    path_safety, settings,
};

#[tauri::command]
pub fn execute_move_plan(
    selected_path: String,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    plans: Vec<MovePlan>,
    app: AppHandle,
) -> Result<(), String> {
    let selected_folder = PathBuf::from(&selected_path);
    path_safety::validate_selected_folder(&selected_folder)?;
    emit_move_progress_message(
        &app,
        MoveProgressPhase::Starting,
        None,
        "Starting move execution.",
        0,
        plans.len() as u64,
    );
    let effective_root =
        settings::resolve_effective_destination_root_for_app(&app, &selected_folder)?;
    let history_run = history::start_history_run_with_destination_root(
        &app,
        &selected_folder,
        Some(&effective_root),
    )?;
    let _ = events::emit_history_index_updated(
        &app,
        HistoryIndexUpdatedEvent {
            run_id: Some(history_run.run.run_id.clone()),
        },
    );

    let request = MoveRunRequest {
        selected_folder,
        destination_root: PathBuf::from(&effective_root.root_path),
        files,
        metadata_results,
        plans,
    };
    tauri::async_runtime::spawn(async move {
        move_run::run_move_plan(&app, request, history_run);
    });

    Ok(())
}
