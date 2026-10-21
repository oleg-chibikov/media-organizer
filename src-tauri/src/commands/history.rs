use tauri::AppHandle;

use crate::{
    events, history,
    models::{HistoryIndexEntry, HistoryIndexUpdatedEvent, HistoryRunDetails},
};

#[tauri::command]
pub fn delete_history_run(run_id: String, app: AppHandle) -> Result<(), String> {
    history::delete_history_run(&app, &run_id)?;
    let _ = events::emit_history_index_updated(
        &app,
        HistoryIndexUpdatedEvent {
            run_id: Some(run_id),
        },
    );
    Ok(())
}

#[tauri::command]
pub fn list_history_runs(app: AppHandle) -> Result<Vec<HistoryIndexEntry>, String> {
    history::list_history_runs(&app)
}

#[tauri::command]
pub fn get_history_run(
    run_id: String,
    app: AppHandle,
) -> Result<Option<HistoryRunDetails>, String> {
    history::get_history_run(&app, &run_id)
}
