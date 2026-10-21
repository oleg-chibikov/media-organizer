use tauri::{AppHandle, State};

use super::app_dirs::resource_dir;
use crate::{
    app_state::AppState,
    models::{MetadataReadRequest, MetadataResult},
};

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
