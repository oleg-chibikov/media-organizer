use tauri::{AppHandle, State};

use super::app_dirs::resource_dir;
use crate::{
    app_state::AppState,
    models::{FileKind, MetadataReadRequest, MetadataResult},
};

#[tauri::command]
pub async fn read_metadata_for_file(
    app: AppHandle,
    state: State<'_, AppState>,
    file_id: String,
    source_path: String,
    file_kind: FileKind,
) -> Result<MetadataResult, String> {
    let request = MetadataReadRequest {
        file_id,
        source_path,
        file_kind,
    };
    state
        .metadata_queue
        .read_one(resource_dir(&app), request)
        .await
}

#[tauri::command]
pub async fn read_metadata_for_files(
    app: AppHandle,
    state: State<'_, AppState>,
    requests: Vec<MetadataReadRequest>,
) -> Result<Vec<MetadataResult>, String> {
    state
        .metadata_queue
        .read_batch(resource_dir(&app), requests)
        .await
}
