use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use super::{background::run_blocking, folder_dialog::pick_folder};
use crate::{app_state::AppState, system_explorer};

#[tauri::command]
pub fn health_check() -> String {
    "ok".to_string()
}

#[tauri::command]
pub async fn choose_folder(app: AppHandle) -> Option<String> {
    let folder = pick_folder(&app).await?;
    app.state::<AppState>()
        .picked_folders
        .remember(Path::new(&folder));
    Some(folder)
}

#[tauri::command]
pub async fn reveal_file_in_explorer(source_path: String) -> Result<(), String> {
    run_blocking(move || {
        system_explorer::reveal_path_in_system_explorer(&PathBuf::from(source_path))
    })
    .await
}

#[tauri::command]
pub async fn open_file(source_path: String) -> Result<(), String> {
    run_blocking(move || system_explorer::open_path_in_default_app(&PathBuf::from(source_path)))
        .await
}
