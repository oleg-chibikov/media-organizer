use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::{
    metadata,
    models::{MetadataResult, MetadataStatus},
    scanner,
};
use tauri_plugin_dialog::DialogExt;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub struct AppState {
    pub scan_cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
}

#[tauri::command]
pub fn health_check() -> String {
    "ok".to_string()
}

#[tauri::command]
pub fn choose_folder(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|path| path.to_string())
}

#[tauri::command]
pub fn start_scan(
    selected_path: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let selected_folder = PathBuf::from(&selected_path);
    validate_folder(&selected_folder)?;

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
pub fn cancel_scan(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let slot = state
        .scan_cancellation_token
        .lock()
        .map_err(|_| "Failed to lock scan cancellation token state".to_string())?;

    if let Some(token) = slot.as_ref() {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub fn read_metadata_for_file(
    app: tauri::AppHandle,
    file_id: String,
    source_path: String,
) -> Result<MetadataResult, String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Ok(MetadataResult {
            file_id,
            chosen_date: None,
            chosen_date_source: None,
            date_kind: None,
            raw_metadata_date: None,
            metadata_status: MetadataStatus::Error,
            error: Some("File no longer exists on disk.".to_string()),
        });
    }

    let raw_dates = match metadata::read_metadata_with_exiftool(&app, &source) {
        Ok(value) => value,
        Err(error) => {
            return Ok(MetadataResult {
                file_id,
                chosen_date: None,
                chosen_date_source: None,
                date_kind: None,
                raw_metadata_date: None,
                metadata_status: MetadataStatus::Error,
                error: Some(error),
            })
        }
    };

    let detected_raw_date = [
        ("SubSecDateTimeOriginal", raw_dates.sub_sec_date_time_original.as_ref()),
        ("DateTimeOriginal", raw_dates.date_time_original.as_ref()),
        ("CreateDate", raw_dates.create_date.as_ref()),
        ("ModifyDate", raw_dates.modify_date.as_ref()),
        ("MediaCreateDate", raw_dates.media_create_date.as_ref()),
        ("TrackCreateDate", raw_dates.track_create_date.as_ref()),
        ("FileModifyDate", raw_dates.file_modify_date.as_ref()),
    ]
    .into_iter()
    .find_map(|(tag, value)| value.map(|raw| (tag.to_string(), raw.to_string())));

    if let Some((source_tag, raw_value)) = detected_raw_date {
        return Ok(MetadataResult {
            file_id,
            chosen_date: None,
            chosen_date_source: Some(source_tag),
            date_kind: None,
            raw_metadata_date: Some(raw_value),
            metadata_status: MetadataStatus::Ready,
            error: None,
        });
    }

    Ok(MetadataResult {
        file_id,
        chosen_date: None,
        chosen_date_source: None,
        date_kind: None,
        raw_metadata_date: None,
        metadata_status: MetadataStatus::NoDate,
        error: None,
    })
}

fn validate_folder(selected_folder: &Path) -> Result<(), String> {
    if !selected_folder.exists() {
        return Err("Selected folder does not exist.".to_string());
    }
    if !selected_folder.is_dir() {
        return Err("Selected path is not a folder.".to_string());
    }

    fs::read_dir(selected_folder)
        .map_err(|error| format!("Selected folder is not readable: {error}"))?;

    Ok(())
}
