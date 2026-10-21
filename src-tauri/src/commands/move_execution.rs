use std::path::PathBuf;

use tauri::AppHandle;

use super::{
    app_dirs::{history_store, settings_store},
    background::{run_blocking, spawn_job},
};
use crate::{
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
    let settings = settings_store(&app)?;
    let history = history_store(&app)?;
    let batch = MoveBatch {
        files,
        metadata_results,
        plans,
    };
    let sink = app.clone();
    let run = run_blocking(move || {
        let selected_folder = PathBuf::from(selected_path);
        move_run::start_move_run(&sink, &settings, &history, &selected_folder, batch)
    })
    .await?;

    spawn_job(move || run.execute(&app));
    Ok(())
}
