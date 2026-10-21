use std::path::PathBuf;

use tauri::{AppHandle, State};
use tokio_util::sync::CancellationToken;

use crate::{app_state::AppState, path_safety, scanner};

#[tauri::command]
pub fn start_scan(
    selected_path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let selected_folder = PathBuf::from(&selected_path);
    path_safety::validate_selected_folder(&selected_folder)?;

    let cancellation_token = CancellationToken::new();
    {
        let mut slot = state
            .scan_cancellation_token
            .lock()
            .map_err(|_| "Failed to lock scan cancellation token state".to_string())?;
        *slot = Some(cancellation_token.clone());
    }

    let state_ref = state.scan_cancellation_token.clone();
    tauri::async_runtime::spawn(async move {
        let _ = scanner::scan_folder(app, selected_folder, cancellation_token);
        if let Ok(mut slot) = state_ref.lock() {
            *slot = None;
        }
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, AppState>) -> Result<(), String> {
    let slot = state
        .scan_cancellation_token
        .lock()
        .map_err(|_| "Failed to lock scan cancellation token state".to_string())?;

    if let Some(token) = slot.as_ref() {
        token.cancel();
    }
    Ok(())
}
