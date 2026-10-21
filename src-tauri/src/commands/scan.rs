use std::path::PathBuf;

use tauri::{AppHandle, State};

use super::background::{run_blocking, spawn_job};
use crate::{app_state::AppState, path_safety, scanner};

#[tauri::command]
pub async fn start_scan(
    selected_path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let selected_folder = PathBuf::from(selected_path);
    let validated_folder = selected_folder.clone();
    run_blocking(move || path_safety::validate_selected_folder(&validated_folder)).await?;

    let active_scan = state.active_scan.clone();
    let ticket = active_scan.begin();
    spawn_job(move || {
        active_scan.run(&ticket, |token| {
            scanner::scan_folder(&app, &selected_folder, token);
        });
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, AppState>) {
    state.active_scan.cancel();
}
