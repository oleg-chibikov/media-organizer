use tauri::AppHandle;

use super::{app_dirs::settings_store, background::run_blocking, session::scan_session};
use crate::{home_location, models::PlaceDetection};

/// The place folder of each photo and video of the scan. Geocoding runs on the blocking pool.
#[tauri::command]
pub async fn detect_file_places(
    scan_id: u64,
    file_ids: Vec<String>,
    app: AppHandle,
) -> Result<PlaceDetection, String> {
    let session = scan_session(&app, scan_id)?;
    let files = session.records(&file_ids)?;
    let metadata_results = session.metadata(&file_ids);
    let settings = settings_store(&app)?;
    run_blocking(move || home_location::detect_file_places(&settings, &files, &metadata_results))
        .await
}
