use std::path::PathBuf;

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::{app_state::AppState, models::ProcessingConfig, system_explorer};

#[tauri::command]
pub fn health_check() -> String {
    "ok".to_string()
}

#[tauri::command]
pub fn get_processing_config(state: State<'_, AppState>) -> ProcessingConfig {
    state.metadata_queue.processing_config()
}

#[tauri::command]
pub fn choose_folder(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|path| path.to_string())
}

#[tauri::command]
pub fn reveal_file_in_explorer(source_path: String) -> Result<(), String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err("File no longer exists on disk.".to_string());
    }

    system_explorer::reveal_path_in_system_explorer(&source)
}
