use tauri::AppHandle;

use super::{app_dirs::history_store, background::run_blocking};
use crate::{
    events,
    models::{HistoryIndexEntry, HistoryRunDetails},
};

#[tauri::command]
pub async fn delete_history_run(run_id: String, app: AppHandle) -> Result<(), String> {
    let store = history_store(&app)?;
    let deleted_run_id = run_id.clone();
    run_blocking(move || store.delete_run(&deleted_run_id)).await?;
    events::history_index_updated(&app, &run_id);
    Ok(())
}

#[tauri::command]
pub async fn list_history_runs(app: AppHandle) -> Result<Vec<HistoryIndexEntry>, String> {
    let store = history_store(&app)?;
    run_blocking(move || store.list_runs()).await
}

#[tauri::command]
pub async fn get_history_run(
    run_id: String,
    app: AppHandle,
) -> Result<Option<HistoryRunDetails>, String> {
    let store = history_store(&app)?;
    run_blocking(move || store.get_run(&run_id)).await
}
