use tauri::{AppHandle, Manager};

use super::{app_dirs::history_store, session::scan_session};
use crate::app_state::AppState;

/// Moves the scan's ready files, then the ones that become ready while it's still checked.
/// Ends with `move:done` once the source folders are cleaned.
#[tauri::command]
pub async fn start_moves(scan_id: u64, app: AppHandle) -> Result<(), String> {
    let session = scan_session(&app, scan_id)?;
    app.state::<AppState>()
        .picked_folders
        .ensure_picked(session.folder())?;
    let history = history_store(&app)?;
    session.start_moves(app.clone(), history)
}

/// Stops the run after the file moving now. Unreached files keep no result.
#[tauri::command]
pub fn pause_moves(scan_id: u64, app: AppHandle) -> Result<(), String> {
    scan_session(&app, scan_id).map(|session| session.pause_moves())
}

#[tauri::command]
pub fn resume_moves(scan_id: u64, app: AppHandle) -> Result<(), String> {
    scan_session(&app, scan_id).map(|session| session.resume_moves())
}
