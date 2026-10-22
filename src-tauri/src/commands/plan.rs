use std::collections::HashMap;

use tauri::AppHandle;

use super::{background::run_blocking, session::scan_session};
use crate::{
    models::{FilePlace, MoveConfirmationSummary},
    mover,
};

/// Plans every file of the scan with a date into its place folder, or a date folder when
/// `file_places` has none for it. Files read later are planned as they come. Plans of a
/// `preview` wait for the final places before any file moves.
#[tauri::command]
pub async fn apply_file_places(
    scan_id: u64,
    file_places: HashMap<String, FilePlace>,
    preview: bool,
    app: AppHandle,
) -> Result<(), String> {
    let session = scan_session(&app, scan_id)?;
    run_blocking(move || {
        session.set_places(&app, file_places, preview);
        Ok(())
    })
    .await
}

/// Drops the scan's plans for a new destination. They come back once places are applied.
#[tauri::command]
pub async fn clear_scan_plans(scan_id: u64, app: AppHandle) -> Result<(), String> {
    let session = scan_session(&app, scan_id)?;
    run_blocking(move || {
        session.clear_plans(&app);
        Ok(())
    })
    .await
}

#[tauri::command]
pub fn get_move_confirmation_summary(
    scan_id: u64,
    file_ids: Vec<String>,
    app: AppHandle,
) -> Result<MoveConfirmationSummary, String> {
    let session = scan_session(&app, scan_id)?;
    let plans = session.plans(&file_ids)?;
    let files = session.records(&file_ids)?;
    let metadata_results = session.metadata(&file_ids);
    Ok(mover::build_move_confirmation_summary(
        &files,
        &metadata_results,
        &plans,
    ))
}
