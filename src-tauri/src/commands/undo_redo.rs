use tauri::AppHandle;

use super::{
    app_dirs::history_store,
    background::{run_blocking, spawn_job},
};
use crate::undo_redo::{self, ReplayDirection};

#[tauri::command]
pub async fn undo_history_run(run_id: String, app: AppHandle) -> Result<(), String> {
    start_replay(app, run_id, ReplayDirection::Undo).await
}

#[tauri::command]
pub async fn redo_history_run(run_id: String, app: AppHandle) -> Result<(), String> {
    start_replay(app, run_id, ReplayDirection::Redo).await
}

async fn start_replay(
    app: AppHandle,
    run_id: String,
    direction: ReplayDirection,
) -> Result<(), String> {
    let store = history_store(&app)?;
    let job = run_blocking(move || undo_redo::prepare_replay(&store, &run_id, direction)).await?;
    spawn_job(move || job.run(&app));
    Ok(())
}
