use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::{app_state::AppState, scan_session::ScanSession};

/// The files and dates of scan `scan_id`, or an error once a newer scan replaced it.
pub(super) fn scan_session(app: &AppHandle, scan_id: u64) -> Result<Arc<ScanSession>, String> {
    app.state::<AppState>().current_scan.session(scan_id)
}
