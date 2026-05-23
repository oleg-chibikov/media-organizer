use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use chrono::Local;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::{
    events, logging, metadata,
    models::{
        FileKind, FileRecord, MetadataResult, MetadataStatus, MoveConfirmationSummary,
        MoveFileEvent, MoveFinishedEvent, MovePlan, MoveResult, MoveResultStatus, ProcessingConfig,
    },
    mover, planner, scanner,
};
use tauri_plugin_dialog::DialogExt;

const MIN_METADATA_CONCURRENCY: usize = 1;
const MAX_METADATA_CONCURRENCY: usize = 8;
const DEFAULT_METADATA_QUEUE_CAPACITY: usize = 256;

pub struct AppState {
    pub scan_cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
    pub metadata_workers: Vec<Arc<Mutex<metadata::ExifToolWorkerManager>>>,
    pub metadata_next_worker: AtomicUsize,
    pub metadata_slots: Arc<Semaphore>,
    pub metadata_waiting: AtomicUsize,
    pub metadata_queue_capacity: usize,
    pub metadata_timeout: Duration,
}

impl Default for AppState {
    fn default() -> Self {
        let default_metadata_concurrency = default_metadata_concurrency();
        let metadata_concurrency = read_env_usize(
            "MEDIA_ORGANIZER_METADATA_CONCURRENCY",
            default_metadata_concurrency,
            MIN_METADATA_CONCURRENCY,
            MAX_METADATA_CONCURRENCY,
        );
        let metadata_queue_capacity = read_env_usize(
            "MEDIA_ORGANIZER_METADATA_QUEUE_CAPACITY",
            DEFAULT_METADATA_QUEUE_CAPACITY,
            metadata_concurrency,
            4096,
        );
        let metadata_timeout_ms = read_env_u64(
            "MEDIA_ORGANIZER_METADATA_TIMEOUT_MS",
            metadata::DEFAULT_REQUEST_TIMEOUT_MS,
            1_000,
            120_000,
        );

        let metadata_workers = (0..metadata_concurrency)
            .map(|_| Arc::new(Mutex::new(metadata::ExifToolWorkerManager::default())))
            .collect();

        Self {
            scan_cancellation_token: Arc::new(Mutex::new(None)),
            metadata_workers,
            metadata_next_worker: AtomicUsize::new(0),
            metadata_slots: Arc::new(Semaphore::new(metadata_concurrency)),
            metadata_waiting: AtomicUsize::new(0),
            metadata_queue_capacity,
            metadata_timeout: Duration::from_millis(metadata_timeout_ms),
        }
    }
}

#[tauri::command]
pub fn health_check() -> String {
    "ok".to_string()
}

#[tauri::command]
pub fn get_processing_config(state: tauri::State<'_, AppState>) -> ProcessingConfig {
    ProcessingConfig {
        metadata_concurrency: state.metadata_workers.len(),
        metadata_queue_capacity: state.metadata_queue_capacity,
        metadata_timeout_ms: state.metadata_timeout.as_millis() as u64,
    }
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
pub async fn read_metadata_for_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    file_id: String,
    source_path: String,
    file_kind: FileKind,
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

    let queued = state.metadata_waiting.fetch_add(1, Ordering::SeqCst) + 1;
    if queued > state.metadata_queue_capacity {
        state.metadata_waiting.fetch_sub(1, Ordering::SeqCst);
        return Ok(MetadataResult {
            file_id,
            chosen_date: None,
            chosen_date_source: None,
            date_kind: None,
            raw_metadata_date: None,
            metadata_status: MetadataStatus::Error,
            error: Some(format!(
                "Metadata queue is full (capacity {}). Try again shortly.",
                state.metadata_queue_capacity
            )),
        });
    }

    let permit = state
        .metadata_slots
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| "Metadata queue is unavailable.".to_string())?;
    state.metadata_waiting.fetch_sub(1, Ordering::SeqCst);

    let worker = select_metadata_worker(&state);
    let timeout = state.metadata_timeout;
    let source_for_worker = source.clone();
    let app_for_worker = app.clone();

    let read_result = tauri::async_runtime::spawn_blocking(move || {
        let mut manager = worker
            .lock()
            .map_err(|_| "Failed to lock ExifTool worker state.".to_string())?;
        manager.read_metadata(&app_for_worker, &source_for_worker, timeout)
    })
    .await
    .map_err(|error| format!("Metadata worker task failed: {error}"))?;

    drop(permit);

    let outcome = match read_result {
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
            });
        }
    };

    let selection = match metadata::select_metadata_date(&file_kind, &outcome.raw_dates) {
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
            });
        }
    };

    if let Some(date) = selection {
        return Ok(MetadataResult {
            file_id,
            chosen_date: Some(date.chosen_date),
            chosen_date_source: Some(date.chosen_date_source),
            date_kind: Some(date.date_kind),
            raw_metadata_date: Some(date.raw_metadata_date),
            metadata_status: MetadataStatus::Ready,
            error: outcome
                .did_restart
                .then(|| "ExifTool worker restarted after a previous crash or stall.".to_string()),
        });
    }

    Ok(MetadataResult {
        file_id,
        chosen_date: None,
        chosen_date_source: None,
        date_kind: None,
        raw_metadata_date: None,
        metadata_status: MetadataStatus::NoDate,
        error: outcome
            .did_restart
            .then(|| "ExifTool worker restarted after a previous crash or stall.".to_string()),
    })
}

#[tauri::command]
pub fn create_move_plan(
    selected_path: String,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let selected_folder = PathBuf::from(&selected_path);
    validate_folder(&selected_folder)?;

    tauri::async_runtime::spawn(async move {
        let _ = planner::stream_move_plan(&app, selected_folder, files, metadata_results);
    });

    Ok(())
}

#[tauri::command]
pub fn get_move_confirmation_summary(
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    plans: Vec<MovePlan>,
) -> MoveConfirmationSummary {
    mover::build_move_confirmation_summary(&files, &metadata_results, &plans)
}

#[tauri::command]
pub fn execute_move_plan(
    selected_path: String,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    plans: Vec<MovePlan>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let selected_folder = PathBuf::from(&selected_path);
    validate_folder(&selected_folder)?;
    let log_file = logging::open_processing_log_for_append(&selected_folder)?;

    tauri::async_runtime::spawn(async move {
        let prepared = mover::prepare_move_operations(&files, &plans);
        let rechecked = mover::recheck_destination_conflicts(prepared.operations);
        let metadata_by_id: HashMap<&str, &MetadataResult> = metadata_results
            .iter()
            .map(|metadata| (metadata.file_id.as_str(), metadata))
            .collect();
        let plans_by_id: HashMap<&str, &MovePlan> = plans
            .iter()
            .map(|plan| (plan.file_id.as_str(), plan))
            .collect();
        let files_by_id: HashMap<&str, &FileRecord> =
            files.iter().map(|file| (file.id.as_str(), file)).collect();
        let mut moved = 0u64;
        let mut skipped = 0u64;
        let mut failed = 0u64;
        let mut log_file = Some(log_file);

        for result in prepared.skipped.into_iter().chain(rechecked.skipped) {
            append_move_result_to_log(
                &mut log_file,
                &result,
                &metadata_by_id,
                &plans_by_id,
                &files_by_id,
            );
            count_move_result(&result, &mut moved, &mut skipped, &mut failed);
            let _ = events::emit_move_file(&app, MoveFileEvent { result });
        }

        for operation in rechecked.ready {
            let result = mover::execute_move_with_rename(&operation);
            append_move_result_to_log(
                &mut log_file,
                &result,
                &metadata_by_id,
                &plans_by_id,
                &files_by_id,
            );
            count_move_result(&result, &mut moved, &mut skipped, &mut failed);
            let _ = events::emit_move_file(&app, MoveFileEvent { result });
        }

        let _ = events::emit_move_finished(
            &app,
            MoveFinishedEvent {
                moved,
                skipped,
                failed,
            },
        );
    });

    Ok(())
}

fn append_move_result_to_log(
    log_file: &mut Option<fs::File>,
    result: &MoveResult,
    metadata_by_id: &HashMap<&str, &MetadataResult>,
    plans_by_id: &HashMap<&str, &MovePlan>,
    files_by_id: &HashMap<&str, &FileRecord>,
) {
    let Some(file) = log_file.as_mut() else {
        return;
    };

    let metadata_date = metadata_by_id
        .get(result.file_id.as_str())
        .and_then(|metadata| metadata.chosen_date.clone());

    let original_name = plans_by_id
        .get(result.file_id.as_str())
        .map(|plan| plan.original_name.clone())
        .or_else(|| {
            files_by_id
                .get(result.file_id.as_str())
                .map(|file| file.original_name.clone())
        })
        .unwrap_or_else(|| result.file_id.clone());

    let record = logging::build_processing_log_record(
        &result.attempted_source_path,
        &result.attempted_destination_path,
        metadata_date,
        original_name,
        result.move_strategy.clone(),
        result.result_status.clone(),
        result.error.clone(),
        Local::now(),
    );

    let _ = logging::append_record(file, &record);
}

fn select_metadata_worker(state: &AppState) -> Arc<Mutex<metadata::ExifToolWorkerManager>> {
    let index = state.metadata_next_worker.fetch_add(1, Ordering::Relaxed);
    let slot = index % state.metadata_workers.len();
    state.metadata_workers[slot].clone()
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

fn default_metadata_concurrency() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(MIN_METADATA_CONCURRENCY, MAX_METADATA_CONCURRENCY)
}

fn read_env_usize(name: &str, default: usize, min: usize, max: usize) -> usize {
    match std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
    {
        Some(value) => value.clamp(min, max),
        None => default,
    }
}

fn read_env_u64(name: &str, default: u64, min: u64, max: u64) -> u64 {
    match std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
    {
        Some(value) => value.clamp(min, max),
        None => default,
    }
}

fn count_move_result(result: &MoveResult, moved: &mut u64, skipped: &mut u64, failed: &mut u64) {
    match result.result_status {
        MoveResultStatus::Moved => *moved += 1,
        MoveResultStatus::Skipped => *skipped += 1,
        MoveResultStatus::Failed => *failed += 1,
    }
}
