use tauri::AppHandle;

use crate::{history, models::HistoryRunDetails, undo_redo};

#[tauri::command]
pub fn undo_history_run(run_id: String, app: AppHandle) -> Result<(), String> {
    let details = find_history_run(&app, &run_id)?;
    let operations = undo_redo::undoable_operations(&details)?;
    let initial_status = details.run.run_status;

    tauri::async_runtime::spawn(async move {
        undo_redo::run_undo(&app, &run_id, initial_status, &operations);
    });

    Ok(())
}

#[tauri::command]
pub fn redo_history_run(run_id: String, app: AppHandle) -> Result<(), String> {
    let details = find_history_run(&app, &run_id)?;
    let operations = undo_redo::redoable_operations(&details)?;
    let initial_status = details.run.run_status;

    tauri::async_runtime::spawn(async move {
        undo_redo::run_redo(&app, &run_id, initial_status, &operations);
    });

    Ok(())
}

fn find_history_run(app: &AppHandle, run_id: &str) -> Result<HistoryRunDetails, String> {
    history::get_history_run(app, run_id)?
        .ok_or_else(|| format!("History run '{run_id}' was not found."))
}
