use std::{path::PathBuf, sync::Arc};

use tauri::{AppHandle, State};

use super::{
    app_dirs::resource_dir,
    background::{run_blocking, spawn_job},
};
use crate::{
    app_state::AppState,
    path_safety, scan_metadata,
    scan_metadata::{QueueReader, ReadOptions},
    scanner,
};

#[tauri::command]
pub async fn start_scan(
    selected_path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let selected_folder = PathBuf::from(selected_path);
    let validated_folder = selected_folder.clone();
    run_blocking(move || path_safety::validate_selected_folder(&validated_folder)).await?;

    let active_scan = state.active_scan.clone();
    let ticket = active_scan.begin();
    let reader = QueueReader {
        queue: state.metadata_queue.clone(),
        resource_dir: resource_dir(&app),
    };
    let feed = scan_metadata::start_reading(
        app.clone(),
        ReadOptions {
            scan_id: ticket.scan_id(),
            token: ticket.token.clone(),
            reader: Arc::new(reader),
            pause: state.metadata_pause.clone(),
            reader_count: state.metadata_queue.concurrency(),
        },
    );
    spawn_job(move || {
        active_scan.run(&ticket, |token| {
            scanner::scan_folder(&app, &selected_folder, token, ticket.scan_id(), |records| {
                feed.push(records)
            });
        });
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, AppState>) {
    state.active_scan.cancel();
}

/// Holds metadata reads while a move run is paused. Reads already running finish.
#[tauri::command]
pub fn set_metadata_paused(paused: bool, state: State<'_, AppState>) {
    state.metadata_pause.set(paused);
}
