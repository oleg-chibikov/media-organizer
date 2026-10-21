use tauri::{AppHandle, State};

use crate::{
    app_state::AppState,
    metadata_queue::metadata_error_result,
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
        file_id: file_id.clone(),
        source_path,
        file_kind,
    };
    let mut results = state.metadata_queue.read_batch(app, vec![request]).await?;
    Ok(results.pop().unwrap_or_else(|| {
        metadata_error_result(file_id, "Metadata worker returned no result.".to_string())
    }))
}

#[tauri::command]
pub async fn read_metadata_for_files(
    app: AppHandle,
    state: State<'_, AppState>,
    requests: Vec<MetadataReadRequest>,
) -> Result<Vec<MetadataResult>, String> {
    state.metadata_queue.read_batch(app, requests).await
}
