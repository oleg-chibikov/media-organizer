use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use chrono::Utc;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::{
    events, history, metadata,
    models::{
        DestinationSettings, EffectiveDestinationRoot, FileKind, FileRecord, HistoryIndexEntry,
        HistoryIndexUpdatedEvent, HistoryOperation, HistoryOperationKind, HistoryOperationStatus,
        HistoryRedoProgressEvent, HistoryRunDetails, HistoryRunStatus, HistoryUndoProgressEvent,
        MetadataReadRequest, MetadataResult, MetadataStatus, MoveConfirmationSummary,
        MoveFileEvent, MoveFinishedEvent, MovePlan, MoveProgressEvent, MoveProgressPhase,
        MoveResult, MoveResultStatus, MoveStrategy, ProcessingConfig,
    },
    mover, planner, scanner, settings,
};
use tauri::{AppHandle, Manager};
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
pub fn get_destination_settings(app: AppHandle) -> Result<DestinationSettings, String> {
    load_destination_settings_for_app(&app)
}

#[tauri::command]
pub fn choose_root_media_folder(app: AppHandle) -> Result<Option<DestinationSettings>, String> {
    let Some(selected_path) = app.dialog().file().blocking_pick_folder() else {
        return Ok(None);
    };

    let root_path = PathBuf::from(selected_path.to_string());
    settings::validate_custom_root_media_folder(&root_path)?;

    let current = load_destination_settings_for_app(&app)?;
    let updated =
        settings::set_root_media_folder(&current, Some(root_path.to_string_lossy().to_string()));
    save_destination_settings_for_app(&app, &updated).map(Some)
}

#[tauri::command]
pub fn clear_root_media_folder(app: AppHandle) -> Result<DestinationSettings, String> {
    let current = load_destination_settings_for_app(&app)?;
    let updated = settings::clear_root_media_folder(&current);
    save_destination_settings_for_app(&app, &updated)
}

#[tauri::command]
pub fn set_force_processed_folder(
    force_processed_folder: bool,
    app: AppHandle,
) -> Result<DestinationSettings, String> {
    let current = load_destination_settings_for_app(&app)?;
    let updated = settings::set_force_processed_folder(&current, force_processed_folder);
    save_destination_settings_for_app(&app, &updated)
}

fn load_destination_settings_for_app(app: &AppHandle) -> Result<DestinationSettings, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve appData directory: {error}"))?;
    settings::load_destination_settings(&app_data_dir)
}

fn save_destination_settings_for_app(
    app: &AppHandle,
    destination_settings: &DestinationSettings,
) -> Result<DestinationSettings, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve appData directory: {error}"))?;
    settings::save_destination_settings(&app_data_dir, destination_settings)
}

fn resolve_effective_destination_root_for_app(
    app: &AppHandle,
    selected_folder: &Path,
) -> Result<EffectiveDestinationRoot, String> {
    let destination_settings = load_destination_settings_for_app(app)?;
    settings::resolve_effective_destination_root(selected_folder, &destination_settings)
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
    let request = MetadataReadRequest {
        file_id: file_id.clone(),
        source_path,
        file_kind,
    };
    let mut results = read_metadata_batch_internal(app, &state, vec![request]).await?;
    Ok(results.pop().unwrap_or_else(|| {
        metadata_error_result(file_id, "Metadata worker returned no result.".to_string())
    }))
}

#[tauri::command]
pub async fn read_metadata_for_files(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    requests: Vec<MetadataReadRequest>,
) -> Result<Vec<MetadataResult>, String> {
    read_metadata_batch_internal(app, &state, requests).await
}

async fn read_metadata_batch_internal(
    app: tauri::AppHandle,
    state: &tauri::State<'_, AppState>,
    requests: Vec<MetadataReadRequest>,
) -> Result<Vec<MetadataResult>, String> {
    if requests.is_empty() {
        return Ok(Vec::new());
    }

    let request_count = requests.len();
    let queued = state
        .metadata_waiting
        .fetch_add(request_count, Ordering::SeqCst)
        + request_count;
    if queued > state.metadata_queue_capacity {
        state
            .metadata_waiting
            .fetch_sub(request_count, Ordering::SeqCst);
        return Ok(requests
            .into_iter()
            .map(|request| {
                metadata_error_result(
                    request.file_id,
                    format!(
                        "Metadata queue is full (capacity {}). Try again shortly.",
                        state.metadata_queue_capacity
                    ),
                )
            })
            .collect());
    }

    let mut results: Vec<Option<MetadataResult>> = vec![None; request_count];
    let mut pending_requests: Vec<(usize, MetadataReadRequest, PathBuf)> = Vec::new();

    for (index, request) in requests.into_iter().enumerate() {
        let source = PathBuf::from(&request.source_path);
        if source.exists() {
            pending_requests.push((index, request, source));
        } else {
            results[index] = Some(metadata_error_result(
                request.file_id,
                "File no longer exists on disk.".to_string(),
            ));
        }
    }

    if pending_requests.is_empty() {
        state
            .metadata_waiting
            .fetch_sub(request_count, Ordering::SeqCst);
        return Ok(unwrap_metadata_results(results));
    }

    let permit = state
        .metadata_slots
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| "Metadata queue is unavailable.".to_string())?;
    state
        .metadata_waiting
        .fetch_sub(request_count, Ordering::SeqCst);

    let worker = select_metadata_worker(state);
    let timeout = metadata_batch_timeout(state.metadata_timeout, pending_requests.len());
    let source_paths: Vec<PathBuf> = pending_requests
        .iter()
        .map(|(_, _, source)| source.clone())
        .collect();
    let app_for_worker = app.clone();

    let read_result = tauri::async_runtime::spawn_blocking(move || {
        let mut manager = worker
            .lock()
            .map_err(|_| "Failed to lock ExifTool worker state.".to_string())?;
        manager.read_metadata_batch(&app_for_worker, &source_paths, timeout)
    })
    .await
    .map_err(|error| format!("Metadata worker task failed: {error}"))?;

    drop(permit);

    match read_result {
        Ok(outcome) => {
            for ((index, request, _), raw_dates) in
                pending_requests.into_iter().zip(outcome.raw_dates)
            {
                results[index] = Some(metadata_result_from_raw(
                    request,
                    raw_dates,
                    outcome.did_restart,
                ));
            }
        }
        Err(error) => {
            for (index, request, _) in pending_requests {
                results[index] = Some(metadata_error_result(request.file_id, error.clone()));
            }
        }
    }

    Ok(unwrap_metadata_results(results))
}

fn metadata_result_from_raw(
    request: MetadataReadRequest,
    raw_dates: metadata::RawMetadataDates,
    did_restart: bool,
) -> MetadataResult {
    let restart_warning = did_restart
        .then(|| "ExifTool worker restarted after a previous crash or stall.".to_string());

    let selection = match metadata::select_metadata_date(&request.file_kind, &raw_dates) {
        Ok(value) => value,
        Err(error) => return metadata_error_result(request.file_id, error),
    };

    if let Some(date) = selection {
        return MetadataResult {
            file_id: request.file_id,
            chosen_date: Some(date.chosen_date),
            chosen_date_source: Some(date.chosen_date_source),
            date_kind: Some(date.date_kind),
            raw_metadata_date: Some(date.raw_metadata_date),
            device_name: raw_dates.device_name,
            metadata_status: MetadataStatus::Ready,
            error: restart_warning,
        };
    }

    MetadataResult {
        file_id: request.file_id,
        chosen_date: None,
        chosen_date_source: None,
        date_kind: None,
        raw_metadata_date: None,
        device_name: raw_dates.device_name,
        metadata_status: MetadataStatus::NoDate,
        error: restart_warning,
    }
}

fn metadata_error_result(file_id: String, error: String) -> MetadataResult {
    MetadataResult {
        file_id,
        chosen_date: None,
        chosen_date_source: None,
        date_kind: None,
        raw_metadata_date: None,
        device_name: None,
        metadata_status: MetadataStatus::Error,
        error: Some(error),
    }
}

fn metadata_batch_timeout(base_timeout: Duration, batch_size: usize) -> Duration {
    let scaled = base_timeout.as_millis().saturating_mul(batch_size as u128);
    Duration::from_millis(scaled.min(120_000) as u64)
}

fn unwrap_metadata_results(results: Vec<Option<MetadataResult>>) -> Vec<MetadataResult> {
    results
        .into_iter()
        .map(|result| {
            result.unwrap_or_else(|| {
                metadata_error_result(
                    String::new(),
                    "Metadata worker did not produce a result.".to_string(),
                )
            })
        })
        .collect()
}

#[tauri::command]
pub fn create_move_plan(
    selected_path: String,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    app: AppHandle,
) -> Result<(), String> {
    let selected_folder = PathBuf::from(&selected_path);
    validate_folder(&selected_folder)?;
    let effective_root = resolve_effective_destination_root_for_app(&app, &selected_folder)?;
    let destination_root = PathBuf::from(effective_root.root_path);

    tauri::async_runtime::spawn(async move {
        let _ = planner::stream_move_plan_with_destination_root(
            &app,
            destination_root,
            files,
            metadata_results,
        );
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
pub fn delete_history_run(run_id: String, app: tauri::AppHandle) -> Result<(), String> {
    history::delete_history_run(&app, &run_id)?;
    let _ = events::emit_history_index_updated(
        &app,
        HistoryIndexUpdatedEvent {
            run_id: Some(run_id),
        },
    );
    Ok(())
}

#[tauri::command]
pub fn list_history_runs(app: tauri::AppHandle) -> Result<Vec<HistoryIndexEntry>, String> {
    history::list_history_runs(&app)
}

#[tauri::command]
pub fn get_history_run(
    run_id: String,
    app: tauri::AppHandle,
) -> Result<Option<HistoryRunDetails>, String> {
    history::get_history_run(&app, &run_id)
}

#[tauri::command]
pub fn undo_history_run(run_id: String, app: tauri::AppHandle) -> Result<(), String> {
    let details = history::get_history_run(&app, &run_id)?
        .ok_or_else(|| format!("History run '{run_id}' was not found."))?;
    if details.unsafe_for_undo_redo {
        return Err("History run is marked unsafe for undo/redo.".to_string());
    }
    if !matches!(
        details.run.run_status,
        HistoryRunStatus::Completed
            | HistoryRunStatus::CompletedWithErrors
            | HistoryRunStatus::Redone
    ) {
        return Err("Undo is only available for completed or redone runs.".to_string());
    }

    let undoable_operations = history::collect_undoable_operations(&details.operations);
    if undoable_operations.is_empty() {
        return Err("No undoable operations were found for this history run.".to_string());
    }

    let initial_status = details.run.run_status.clone();
    let run_id_for_task = run_id.clone();
    tauri::async_runtime::spawn(async move {
        let summary = execute_undo_operations(&app, &run_id_for_task, &undoable_operations);

        let final_status = undo_completion_status(initial_status, &summary);
        if final_status != details.run.run_status {
            let _ = history::update_history_run_status(&app, &run_id_for_task, final_status);
        }

        let _ = events::emit_history_index_updated(
            &app,
            HistoryIndexUpdatedEvent {
                run_id: Some(run_id_for_task),
            },
        );
    });

    Ok(())
}

#[tauri::command]
pub fn redo_history_run(run_id: String, app: tauri::AppHandle) -> Result<(), String> {
    let details = history::get_history_run(&app, &run_id)?
        .ok_or_else(|| format!("History run '{run_id}' was not found."))?;
    if details.unsafe_for_undo_redo {
        return Err("History run is marked unsafe for undo/redo.".to_string());
    }
    if !matches!(
        details.run.run_status,
        HistoryRunStatus::Undone | HistoryRunStatus::PartiallyUndone
    ) {
        return Err("Redo is only available for undone runs.".to_string());
    }

    let redoable_operations = history::collect_redoable_operations(&details.operations);
    if redoable_operations.is_empty() {
        return Err("No redoable operations were found for this history run.".to_string());
    }

    let initial_status = details.run.run_status.clone();
    let run_id_for_task = run_id.clone();
    tauri::async_runtime::spawn(async move {
        let summary = execute_redo_operations(&app, &run_id_for_task, &redoable_operations);

        let final_status = redo_completion_status(initial_status, &summary);
        if final_status != details.run.run_status {
            let _ = history::update_history_run_status(&app, &run_id_for_task, final_status);
        }

        let _ = events::emit_history_index_updated(
            &app,
            HistoryIndexUpdatedEvent {
                run_id: Some(run_id_for_task),
            },
        );
    });

    Ok(())
}

#[tauri::command]
pub fn execute_move_plan(
    selected_path: String,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    plans: Vec<MovePlan>,
    app: AppHandle,
) -> Result<(), String> {
    let selected_folder = PathBuf::from(&selected_path);
    validate_folder(&selected_folder)?;
    let total_operations = plans.len() as u64;
    emit_move_progress_message(
        &app,
        MoveProgressPhase::Starting,
        None,
        "Starting move execution.",
        0,
        total_operations,
    );
    let effective_root = resolve_effective_destination_root_for_app(&app, &selected_folder)?;
    let destination_root = PathBuf::from(&effective_root.root_path);
    let mut history_run = history::start_history_run_with_destination_root(
        &app,
        &selected_folder,
        Some(&effective_root),
    )?;
    let _ = events::emit_history_index_updated(
        &app,
        HistoryIndexUpdatedEvent {
            run_id: Some(history_run.run.run_id.clone()),
        },
    );

    tauri::async_runtime::spawn(async move {
        emit_move_progress_message(
            &app,
            MoveProgressPhase::Preparing,
            None,
            "Preparing move operations.",
            0,
            total_operations,
        );
        let prepared = mover::prepare_move_operations(&files, &plans);
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
        let mut expected_source_sizes: HashMap<&str, Option<u64>> = HashMap::new();
        let mut moved = 0u64;
        let mut copied = 0u64;
        let mut skipped = 0u64;
        let mut failed = 0u64;
        let mut completed_operations = 0u64;
        let mut results = Vec::new();
        let mut moved_source_directories = Vec::new();
        let mut history_log_had_error = false;

        for result in prepared.skipped {
            ensure_expected_source_size(&result, &files_by_id, &mut expected_source_sizes);
            if append_move_result_to_history(
                &mut history_run,
                &result,
                &metadata_by_id,
                &plans_by_id,
                &files_by_id,
                &expected_source_sizes,
            )
            .is_err()
            {
                history_log_had_error = true;
            }
            count_move_result(&result, &mut moved, &mut copied, &mut skipped, &mut failed);
            completed_operations += 1;
            results.push(result.clone());
            let _ = events::emit_move_file(&app, MoveFileEvent { result });
        }

        for operation in prepared.operations {
            emit_move_progress_message(
                &app,
                MoveProgressPhase::Preparing,
                Some(&operation.file_id),
                "Checking source and destination.",
                completed_operations,
                total_operations,
            );
            let safety_checked = mover::validate_move_safety_with_roots(
                &selected_folder,
                &destination_root,
                vec![operation],
            );

            for result in safety_checked.skipped {
                ensure_expected_source_size(&result, &files_by_id, &mut expected_source_sizes);
                if append_move_result_to_history(
                    &mut history_run,
                    &result,
                    &metadata_by_id,
                    &plans_by_id,
                    &files_by_id,
                    &expected_source_sizes,
                )
                .is_err()
                {
                    history_log_had_error = true;
                }
                count_move_result(&result, &mut moved, &mut copied, &mut skipped, &mut failed);
                completed_operations += 1;
                results.push(result.clone());
                let _ = events::emit_move_file(&app, MoveFileEvent { result });
            }

            let rechecked = mover::recheck_destination_conflicts(safety_checked.ready);
            for result in rechecked.skipped {
                ensure_expected_source_size(&result, &files_by_id, &mut expected_source_sizes);
                if append_move_result_to_history(
                    &mut history_run,
                    &result,
                    &metadata_by_id,
                    &plans_by_id,
                    &files_by_id,
                    &expected_source_sizes,
                )
                .is_err()
                {
                    history_log_had_error = true;
                }
                count_move_result(&result, &mut moved, &mut copied, &mut skipped, &mut failed);
                completed_operations += 1;
                results.push(result.clone());
                let _ = events::emit_move_file(&app, MoveFileEvent { result });
            }

            for operation in rechecked.ready {
                emit_move_progress_message(
                    &app,
                    MoveProgressPhase::Moving,
                    Some(&operation.file_id),
                    "Moving file.",
                    completed_operations,
                    total_operations,
                );
                let source_parent = Path::new(&operation.source_path)
                    .parent()
                    .map(Path::to_path_buf);
                let result = mover::execute_move_with_rename(&operation);
                if matches!(result.result_status, MoveResultStatus::Moved) {
                    if let Some(parent) = source_parent {
                        moved_source_directories.push(parent);
                    }
                }
                ensure_expected_source_size(&result, &files_by_id, &mut expected_source_sizes);
                if append_move_result_to_history(
                    &mut history_run,
                    &result,
                    &metadata_by_id,
                    &plans_by_id,
                    &files_by_id,
                    &expected_source_sizes,
                )
                .is_err()
                {
                    history_log_had_error = true;
                }
                count_move_result(&result, &mut moved, &mut copied, &mut skipped, &mut failed);
                completed_operations += 1;
                results.push(result.clone());
                let _ = events::emit_move_file(&app, MoveFileEvent { result });
            }
        }

        emit_move_progress_message(
            &app,
            MoveProgressPhase::Finalizing,
            None,
            "Finalizing history and cleaning up source folders.",
            completed_operations,
            total_operations,
        );
        cleanup_empty_source_directories(&selected_folder, &moved_source_directories);

        let run_status = history_run_status_for_completion(failed, history_log_had_error);
        let history_run_id = history_run.run.run_id.clone();
        let _ = history::complete_history_run(&mut history_run, run_status);
        let _ = events::emit_history_index_updated(
            &app,
            HistoryIndexUpdatedEvent {
                run_id: Some(history_run_id),
            },
        );

        let _ = events::emit_move_finished(
            &app,
            MoveFinishedEvent {
                moved,
                copied,
                skipped,
                failed,
                results,
            },
        );
    });

    Ok(())
}

#[derive(Debug, Clone)]
struct UndoExecutionSummary {
    attempted: u64,
    succeeded: u64,
    blocked: u64,
    failed: u64,
}

fn execute_undo_operations(
    app: &tauri::AppHandle,
    run_id: &str,
    operations: &[HistoryOperation],
) -> UndoExecutionSummary {
    let total_operations = operations.len() as u64;
    let mut completed_operations = 0u64;
    let mut summary = UndoExecutionSummary {
        attempted: total_operations,
        succeeded: 0,
        blocked: 0,
        failed: 0,
    };

    for operation in operations {
        completed_operations += 1;
        let execution_result = execute_undo_operation(operation);
        let history_operation = build_undo_history_operation(run_id, operation, &execution_result);
        let _ = history::append_history_operation_for_run(app, run_id, history_operation.clone());

        match history_operation.status {
            HistoryOperationStatus::Succeeded => summary.succeeded += 1,
            HistoryOperationStatus::Blocked => summary.blocked += 1,
            HistoryOperationStatus::Failed => summary.failed += 1,
            HistoryOperationStatus::Skipped => {}
        }

        let _ = events::emit_history_undo_progress(
            app,
            HistoryUndoProgressEvent {
                run_id: run_id.to_string(),
                total_operations,
                completed_operations,
                operation: history_operation,
            },
        );
    }

    summary
}

#[derive(Debug, Clone)]
struct UndoExecutionResult {
    status: HistoryOperationStatus,
    error: Option<String>,
}

fn execute_undo_operation(operation: &HistoryOperation) -> UndoExecutionResult {
    match operation.move_strategy {
        MoveStrategy::Rename => undo_rename_operation(operation),
        MoveStrategy::CopyKeepSource => undo_copy_keep_source_operation(operation),
        MoveStrategy::NotAttempted => UndoExecutionResult {
            status: HistoryOperationStatus::Blocked,
            error: Some("Operation strategy is not undoable.".to_string()),
        },
    }
}

fn undo_rename_operation(operation: &HistoryOperation) -> UndoExecutionResult {
    let undo_source = Path::new(&operation.destination_path);
    let undo_destination = Path::new(&operation.source_path);

    if !undo_source.is_file() {
        return UndoExecutionResult {
            status: HistoryOperationStatus::Blocked,
            error: Some("Undo blocked: processed destination file is missing.".to_string()),
        };
    }

    if undo_destination.exists() {
        return UndoExecutionResult {
            status: HistoryOperationStatus::Blocked,
            error: Some("Undo blocked: original source path already exists.".to_string()),
        };
    }

    if let Some(expected_size) = operation.expected_destination_size {
        match fs::metadata(undo_source) {
            Ok(meta) if meta.len() == expected_size => {}
            Ok(meta) => {
                return UndoExecutionResult {
                    status: HistoryOperationStatus::Blocked,
                    error: Some(format!(
                        "Undo blocked: destination size mismatch (expected {expected_size}, found {}).",
                        meta.len()
                    )),
                };
            }
            Err(error) => {
                return UndoExecutionResult {
                    status: HistoryOperationStatus::Blocked,
                    error: Some(format!(
                        "Undo blocked: cannot read destination metadata: {error}"
                    )),
                };
            }
        }
    }

    if let Some(parent) = undo_destination.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return UndoExecutionResult {
                status: HistoryOperationStatus::Failed,
                error: Some(format!(
                    "Undo failed: could not create source parent directory: {error}"
                )),
            };
        }
    }

    match fs::rename(undo_source, undo_destination) {
        Ok(()) => UndoExecutionResult {
            status: HistoryOperationStatus::Succeeded,
            error: None,
        },
        Err(error) => UndoExecutionResult {
            status: HistoryOperationStatus::Failed,
            error: Some(format!(
                "Undo failed: could not move file back to source: {error}"
            )),
        },
    }
}

fn undo_copy_keep_source_operation(operation: &HistoryOperation) -> UndoExecutionResult {
    let copied_destination = Path::new(&operation.destination_path);
    let original_source = Path::new(&operation.source_path);

    if !original_source.is_file() {
        return UndoExecutionResult {
            status: HistoryOperationStatus::Blocked,
            error: Some("Undo blocked: original source file is missing.".to_string()),
        };
    }

    if !copied_destination.is_file() {
        return UndoExecutionResult {
            status: HistoryOperationStatus::Blocked,
            error: Some("Undo blocked: copied destination file is missing.".to_string()),
        };
    }

    if let Some(expected_size) = operation.expected_source_size {
        match fs::metadata(original_source) {
            Ok(meta) if meta.len() == expected_size => {}
            Ok(meta) => {
                return UndoExecutionResult {
                    status: HistoryOperationStatus::Blocked,
                    error: Some(format!(
                        "Undo blocked: source size mismatch (expected {expected_size}, found {}).",
                        meta.len()
                    )),
                };
            }
            Err(error) => {
                return UndoExecutionResult {
                    status: HistoryOperationStatus::Blocked,
                    error: Some(format!(
                        "Undo blocked: cannot read source metadata: {error}"
                    )),
                };
            }
        }
    }

    if let Some(expected_size) = operation.expected_destination_size {
        match fs::metadata(copied_destination) {
            Ok(meta) if meta.len() == expected_size => {}
            Ok(meta) => {
                return UndoExecutionResult {
                    status: HistoryOperationStatus::Blocked,
                    error: Some(format!(
                        "Undo blocked: destination size mismatch (expected {expected_size}, found {}).",
                        meta.len()
                    )),
                };
            }
            Err(error) => {
                return UndoExecutionResult {
                    status: HistoryOperationStatus::Blocked,
                    error: Some(format!(
                        "Undo blocked: cannot read destination metadata: {error}"
                    )),
                };
            }
        }
    }

    match fs::remove_file(copied_destination) {
        Ok(()) => UndoExecutionResult {
            status: HistoryOperationStatus::Succeeded,
            error: None,
        },
        Err(error) => UndoExecutionResult {
            status: HistoryOperationStatus::Failed,
            error: Some(format!(
                "Undo failed: could not remove copied destination: {error}"
            )),
        },
    }
}

fn build_undo_history_operation(
    run_id: &str,
    process_operation: &HistoryOperation,
    result: &UndoExecutionResult,
) -> HistoryOperation {
    let expected_destination_size = match process_operation.move_strategy {
        MoveStrategy::Rename => process_operation.expected_source_size,
        MoveStrategy::CopyKeepSource | MoveStrategy::NotAttempted => None,
    };

    HistoryOperation {
        operation_id: history::next_operation_id(run_id),
        run_id: run_id.to_string(),
        file_id: process_operation.file_id.clone(),
        operation_kind: HistoryOperationKind::Undo,
        source_path: process_operation.destination_path.clone(),
        destination_path: process_operation.source_path.clone(),
        expected_source_size: process_operation.expected_destination_size,
        expected_destination_size,
        metadata_date: process_operation.metadata_date.clone(),
        original_name: process_operation.original_name.clone(),
        move_strategy: process_operation.move_strategy.clone(),
        status: result.status.clone(),
        error: result.error.clone(),
        timestamp: Utc::now().to_rfc3339(),
    }
}

fn undo_completion_status(
    initial_status: HistoryRunStatus,
    summary: &UndoExecutionSummary,
) -> HistoryRunStatus {
    if summary.succeeded == 0 {
        return initial_status;
    }
    if summary.succeeded == summary.attempted {
        return HistoryRunStatus::Undone;
    }
    HistoryRunStatus::PartiallyUndone
}

#[derive(Debug, Clone)]
struct RedoExecutionSummary {
    attempted: u64,
    succeeded: u64,
    blocked: u64,
    failed: u64,
}

fn execute_redo_operations(
    app: &tauri::AppHandle,
    run_id: &str,
    operations: &[HistoryOperation],
) -> RedoExecutionSummary {
    let total_operations = operations.len() as u64;
    let mut completed_operations = 0u64;
    let mut summary = RedoExecutionSummary {
        attempted: total_operations,
        succeeded: 0,
        blocked: 0,
        failed: 0,
    };

    for operation in operations {
        completed_operations += 1;
        let execution_result = execute_redo_operation(operation);
        let history_operation = build_redo_history_operation(run_id, operation, &execution_result);
        let _ = history::append_history_operation_for_run(app, run_id, history_operation.clone());

        match history_operation.status {
            HistoryOperationStatus::Succeeded => summary.succeeded += 1,
            HistoryOperationStatus::Blocked => summary.blocked += 1,
            HistoryOperationStatus::Failed => summary.failed += 1,
            HistoryOperationStatus::Skipped => {}
        }

        let _ = events::emit_history_redo_progress(
            app,
            HistoryRedoProgressEvent {
                run_id: run_id.to_string(),
                total_operations,
                completed_operations,
                operation: history_operation,
            },
        );
    }

    summary
}

#[derive(Debug, Clone)]
struct RedoExecutionResult {
    status: HistoryOperationStatus,
    error: Option<String>,
}

fn execute_redo_operation(operation: &HistoryOperation) -> RedoExecutionResult {
    match operation.move_strategy {
        MoveStrategy::Rename => redo_rename_operation(operation),
        MoveStrategy::CopyKeepSource => redo_copy_keep_source_operation(operation),
        MoveStrategy::NotAttempted => RedoExecutionResult {
            status: HistoryOperationStatus::Blocked,
            error: Some("Operation strategy is not redoable.".to_string()),
        },
    }
}

fn redo_rename_operation(operation: &HistoryOperation) -> RedoExecutionResult {
    let (redo_source, redo_destination) = redo_paths_for_operation(operation);

    if !redo_source.is_file() {
        return RedoExecutionResult {
            status: HistoryOperationStatus::Blocked,
            error: Some("Redo blocked: source file is missing.".to_string()),
        };
    }

    if redo_destination.exists() {
        return RedoExecutionResult {
            status: HistoryOperationStatus::Blocked,
            error: Some("Redo blocked: recorded destination path already exists.".to_string()),
        };
    }

    if let Some(expected_size) = expected_redo_source_size(operation) {
        match fs::metadata(redo_source) {
            Ok(meta) if meta.len() == expected_size => {}
            Ok(meta) => {
                return RedoExecutionResult {
                    status: HistoryOperationStatus::Blocked,
                    error: Some(format!(
                        "Redo blocked: source size mismatch (expected {expected_size}, found {}).",
                        meta.len()
                    )),
                };
            }
            Err(error) => {
                return RedoExecutionResult {
                    status: HistoryOperationStatus::Blocked,
                    error: Some(format!(
                        "Redo blocked: cannot read source metadata: {error}"
                    )),
                };
            }
        }
    }

    if let Some(parent) = redo_destination.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return RedoExecutionResult {
                status: HistoryOperationStatus::Failed,
                error: Some(format!(
                    "Redo failed: could not create destination parent directory: {error}"
                )),
            };
        }
    }

    match fs::rename(redo_source, redo_destination) {
        Ok(()) => RedoExecutionResult {
            status: HistoryOperationStatus::Succeeded,
            error: None,
        },
        Err(error) => RedoExecutionResult {
            status: HistoryOperationStatus::Failed,
            error: Some(format!(
                "Redo failed: could not move file back to processed destination: {error}"
            )),
        },
    }
}

fn redo_copy_keep_source_operation(operation: &HistoryOperation) -> RedoExecutionResult {
    let (redo_source, redo_destination) = redo_paths_for_operation(operation);

    if !redo_source.is_file() {
        return RedoExecutionResult {
            status: HistoryOperationStatus::Blocked,
            error: Some("Redo blocked: source file is missing.".to_string()),
        };
    }

    if redo_destination.exists() {
        return RedoExecutionResult {
            status: HistoryOperationStatus::Blocked,
            error: Some("Redo blocked: recorded destination path already exists.".to_string()),
        };
    }

    if let Some(expected_size) = expected_redo_source_size(operation) {
        match fs::metadata(redo_source) {
            Ok(meta) if meta.len() == expected_size => {}
            Ok(meta) => {
                return RedoExecutionResult {
                    status: HistoryOperationStatus::Blocked,
                    error: Some(format!(
                        "Redo blocked: source size mismatch (expected {expected_size}, found {}).",
                        meta.len()
                    )),
                };
            }
            Err(error) => {
                return RedoExecutionResult {
                    status: HistoryOperationStatus::Blocked,
                    error: Some(format!(
                        "Redo blocked: cannot read source metadata: {error}"
                    )),
                };
            }
        }
    }

    if let Some(parent) = redo_destination.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return RedoExecutionResult {
                status: HistoryOperationStatus::Failed,
                error: Some(format!(
                    "Redo failed: could not create destination parent directory: {error}"
                )),
            };
        }
    }

    match fs::copy(redo_source, redo_destination) {
        Ok(_) => RedoExecutionResult {
            status: HistoryOperationStatus::Succeeded,
            error: None,
        },
        Err(error) => RedoExecutionResult {
            status: HistoryOperationStatus::Failed,
            error: Some(format!(
                "Redo failed: could not copy file to recorded destination: {error}"
            )),
        },
    }
}

fn expected_redo_source_size(operation: &HistoryOperation) -> Option<u64> {
    match operation.operation_kind {
        HistoryOperationKind::Process => operation
            .expected_source_size
            .or(operation.expected_destination_size),
        _ => operation
            .expected_destination_size
            .or(operation.expected_source_size),
    }
}

fn redo_paths_for_operation(operation: &HistoryOperation) -> (&Path, &Path) {
    match operation.operation_kind {
        HistoryOperationKind::Process => (
            Path::new(&operation.source_path),
            Path::new(&operation.destination_path),
        ),
        _ => (
            Path::new(&operation.destination_path),
            Path::new(&operation.source_path),
        ),
    }
}

fn build_redo_history_operation(
    run_id: &str,
    operation: &HistoryOperation,
    result: &RedoExecutionResult,
) -> HistoryOperation {
    let expected_source_size = expected_redo_source_size(operation);
    let (source_path, destination_path) = match operation.operation_kind {
        HistoryOperationKind::Process => (
            operation.source_path.clone(),
            operation.destination_path.clone(),
        ),
        _ => (
            operation.destination_path.clone(),
            operation.source_path.clone(),
        ),
    };

    HistoryOperation {
        operation_id: history::next_operation_id(run_id),
        run_id: run_id.to_string(),
        file_id: operation.file_id.clone(),
        operation_kind: HistoryOperationKind::Redo,
        source_path,
        destination_path,
        expected_source_size,
        expected_destination_size: expected_source_size,
        metadata_date: operation.metadata_date.clone(),
        original_name: operation.original_name.clone(),
        move_strategy: operation.move_strategy.clone(),
        status: result.status.clone(),
        error: result.error.clone(),
        timestamp: Utc::now().to_rfc3339(),
    }
}

fn redo_completion_status(
    initial_status: HistoryRunStatus,
    summary: &RedoExecutionSummary,
) -> HistoryRunStatus {
    if summary.succeeded == 0 {
        return initial_status;
    }
    if summary.succeeded == summary.attempted {
        return HistoryRunStatus::Redone;
    }
    HistoryRunStatus::PartiallyUndone
}

#[tauri::command]
pub fn reveal_file_in_explorer(source_path: String) -> Result<(), String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err("File no longer exists on disk.".to_string());
    }

    reveal_path_in_system_explorer(&source)
}

fn append_move_result_to_history(
    history_run: &mut history::ActiveHistoryRun,
    result: &MoveResult,
    metadata_by_id: &HashMap<&str, &MetadataResult>,
    plans_by_id: &HashMap<&str, &MovePlan>,
    files_by_id: &HashMap<&str, &FileRecord>,
    expected_source_sizes: &HashMap<&str, Option<u64>>,
) -> Result<(), String> {
    let operation = build_history_operation_for_result(
        &history_run.run.run_id,
        result,
        metadata_by_id,
        plans_by_id,
        files_by_id,
        expected_source_sizes,
    );

    history::append_history_operation(history_run, operation)
}

fn ensure_expected_source_size<'a>(
    result: &MoveResult,
    files_by_id: &HashMap<&'a str, &'a FileRecord>,
    expected_source_sizes: &mut HashMap<&'a str, Option<u64>>,
) {
    let Some(file) = files_by_id.get(result.file_id.as_str()) else {
        return;
    };

    expected_source_sizes
        .entry(file.id.as_str())
        .or_insert_with(|| fs::metadata(&file.source_path).ok().map(|meta| meta.len()));
}

fn build_history_operation_for_result(
    run_id: &str,
    result: &MoveResult,
    metadata_by_id: &HashMap<&str, &MetadataResult>,
    plans_by_id: &HashMap<&str, &MovePlan>,
    files_by_id: &HashMap<&str, &FileRecord>,
    expected_source_sizes: &HashMap<&str, Option<u64>>,
) -> HistoryOperation {
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
    let expected_source_size = expected_source_sizes
        .get(result.file_id.as_str())
        .copied()
        .flatten();

    HistoryOperation {
        operation_id: history::next_operation_id(run_id),
        run_id: run_id.to_string(),
        file_id: result.file_id.clone(),
        operation_kind: HistoryOperationKind::Process,
        source_path: result.attempted_source_path.clone(),
        destination_path: result.attempted_destination_path.clone(),
        expected_source_size,
        expected_destination_size: expected_destination_size_from_result(result),
        metadata_date,
        original_name,
        move_strategy: result.move_strategy.clone(),
        status: history_operation_status_for_move_result(&result.result_status),
        error: result.error.clone(),
        timestamp: Utc::now().to_rfc3339(),
    }
}

fn expected_destination_size_from_result(result: &MoveResult) -> Option<u64> {
    match result.result_status {
        MoveResultStatus::Moved | MoveResultStatus::Copied => {
            fs::metadata(&result.attempted_destination_path)
                .ok()
                .map(|meta| meta.len())
        }
        MoveResultStatus::Skipped | MoveResultStatus::Failed => None,
    }
}

fn history_operation_status_for_move_result(
    result_status: &MoveResultStatus,
) -> HistoryOperationStatus {
    match result_status {
        MoveResultStatus::Moved | MoveResultStatus::Copied => HistoryOperationStatus::Succeeded,
        MoveResultStatus::Skipped => HistoryOperationStatus::Skipped,
        MoveResultStatus::Failed => HistoryOperationStatus::Failed,
    }
}

fn history_run_status_for_completion(failed: u64, history_log_had_error: bool) -> HistoryRunStatus {
    if history_log_had_error {
        return HistoryRunStatus::Corrupt;
    }
    if failed > 0 {
        return HistoryRunStatus::CompletedWithErrors;
    }
    HistoryRunStatus::Completed
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
    if path_contains_processed_segment(selected_folder) {
        return Err("Selected folder must not be inside a Processed path.".to_string());
    }

    fs::read_dir(selected_folder)
        .map_err(|error| format!("Selected folder is not readable: {error}"))?;

    Ok(())
}

fn path_contains_processed_segment(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case("Processed")
    })
}

fn reveal_path_in_system_explorer(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let target = path
            .canonicalize()
            .map_err(|error| format!("Failed to resolve file path: {error}"))?;
        Command::new("explorer")
            .arg("/select,")
            .arg(strip_windows_verbatim_prefix(&target))
            .spawn()
            .map_err(|error| format!("Failed to open File Explorer: {error}"))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .map_err(|error| format!("Failed to open Finder: {error}"))?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let parent = path
            .parent()
            .ok_or_else(|| "File has no parent directory.".to_string())?;

        let attempted_select = Command::new("dbus-send")
            .arg("--session")
            .arg("--dest=org.freedesktop.FileManager1")
            .arg("--type=method_call")
            .arg("/org/freedesktop/FileManager1")
            .arg("org.freedesktop.FileManager1.ShowItems")
            .arg(format!(
                "array:string:file://{}",
                path.to_string_lossy().replace(' ', "%20")
            ))
            .arg("string:")
            .spawn();

        if attempted_select.is_ok() {
            return Ok(());
        }

        Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|error| format!("Failed to open file manager: {error}"))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("Reveal in system explorer is not supported on this platform.".to_string())
}

#[cfg(target_os = "windows")]
fn strip_windows_verbatim_prefix(path: &Path) -> PathBuf {
    let path_value = path.to_string_lossy();

    if let Some(stripped) = path_value.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{stripped}"));
    }

    if let Some(stripped) = path_value.strip_prefix(r"\\?\") {
        return PathBuf::from(stripped);
    }

    path.to_path_buf()
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

fn count_move_result(
    result: &MoveResult,
    moved: &mut u64,
    copied: &mut u64,
    skipped: &mut u64,
    failed: &mut u64,
) {
    match result.result_status {
        MoveResultStatus::Moved => *moved += 1,
        MoveResultStatus::Copied => *copied += 1,
        MoveResultStatus::Skipped => *skipped += 1,
        MoveResultStatus::Failed => *failed += 1,
    }
}

fn emit_move_progress_message(
    app: &AppHandle,
    phase: MoveProgressPhase,
    file_id: Option<&str>,
    message: &str,
    completed: u64,
    total: u64,
) {
    let _ = events::emit_move_progress(
        app,
        MoveProgressEvent {
            phase,
            file_id: file_id.map(str::to_string),
            message: message.to_string(),
            completed,
            total,
        },
    );
}

fn cleanup_empty_source_directories(selected_root: &Path, source_directories: &[PathBuf]) {
    let mut visited = HashSet::new();
    let mut cleaned_any_directory = false;

    for directory in source_directories {
        let key = normalized_path_string(directory);
        if visited.insert(key) {
            prune_empty_directory_chain(selected_root, directory);
            cleaned_any_directory = true;
        }
    }

    if cleaned_any_directory {
        remove_empty_selected_root(selected_root);
    }
}

fn prune_empty_directory_chain(selected_root: &Path, start_directory: &Path) {
    let mut current = start_directory.to_path_buf();

    loop {
        if paths_equal_case_insensitive(&current, selected_root)
            || !path_is_within_case_insensitive(&current, selected_root)
        {
            break;
        }

        if !current.is_dir() {
            break;
        }

        let is_empty = match fs::read_dir(&current) {
            Ok(mut entries) => entries.next().is_none(),
            Err(_) => false,
        };

        if !is_empty {
            break;
        }

        if fs::remove_dir(&current).is_err() {
            break;
        }

        let Some(parent) = current.parent() else {
            break;
        };
        current = parent.to_path_buf();
    }
}

fn remove_empty_selected_root(selected_root: &Path) {
    if !selected_root.is_dir() {
        return;
    }

    let is_empty = match fs::read_dir(selected_root) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false,
    };

    if is_empty {
        let _ = fs::remove_dir(selected_root);
    }
}

fn path_is_within_case_insensitive(candidate: &Path, parent: &Path) -> bool {
    let candidate = normalized_path_string(candidate);
    let parent = normalized_path_string(parent);

    candidate == parent || candidate.starts_with(&format!("{parent}/"))
}

fn paths_equal_case_insensitive(left: &Path, right: &Path) -> bool {
    normalized_path_string(left) == normalized_path_string(right)
}

fn normalized_path_string(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs};

    use tempfile::tempdir;

    use super::{
        build_history_operation_for_result, build_redo_history_operation,
        build_undo_history_operation, cleanup_empty_source_directories, execute_redo_operation,
        execute_undo_operation, history_operation_status_for_move_result,
        history_run_status_for_completion,
    };
    use crate::models::{
        FileKind, FileRecord, HistoryOperation, HistoryOperationKind, HistoryOperationStatus,
        HistoryRunStatus, MetadataResult, MetadataStatus, MovePlan, MoveResult, MoveResultStatus,
        MoveStrategy, PlanStatus, ScanStatus,
    };

    #[test]
    fn removes_empty_source_directory_chain_after_moves() {
        let temp = tempdir().expect("temp dir");
        let selected_root = temp.path().join("todo");
        let nested = selected_root.join("a").join("b");
        fs::create_dir_all(&nested).expect("create nested source dirs");

        cleanup_empty_source_directories(&selected_root, &[nested.clone()]);

        assert!(!nested.exists());
        assert!(!selected_root.join("a").exists());
        assert!(!selected_root.exists());
    }

    #[test]
    fn stops_pruning_when_parent_contains_files() {
        let temp = tempdir().expect("temp dir");
        let selected_root = temp.path().join("todo");
        let parent = selected_root.join("a");
        let nested = parent.join("b");
        fs::create_dir_all(&nested).expect("create nested source dirs");
        fs::write(parent.join("keep.txt"), b"keep").expect("write retained file");

        cleanup_empty_source_directories(&selected_root, &[nested.clone()]);

        assert!(!nested.exists());
        assert!(parent.exists());
    }

    #[test]
    fn removes_selected_root_when_processing_leaves_it_empty() {
        let temp = tempdir().expect("temp dir");
        let selected_root = temp.path().join("todo");
        fs::create_dir_all(&selected_root).expect("create selected root");

        cleanup_empty_source_directories(&selected_root, &[selected_root.to_path_buf()]);

        assert!(!selected_root.exists());
    }

    #[test]
    fn keeps_selected_root_when_it_still_contains_files() {
        let temp = tempdir().expect("temp dir");
        let selected_root = temp.path().join("todo");
        fs::create_dir_all(&selected_root).expect("create selected root");
        fs::write(selected_root.join("keep.txt"), b"keep").expect("write retained file");

        cleanup_empty_source_directories(&selected_root, &[selected_root.to_path_buf()]);

        assert!(selected_root.exists());
    }

    #[test]
    fn builds_processing_history_operation_with_required_fields() {
        let temp = tempdir().expect("temp dir");
        let destination_path = temp.path().join("processed.jpg");
        fs::write(&destination_path, b"data").expect("write destination");

        let result = MoveResult {
            file_id: "photo-1".to_string(),
            attempted_source_path: r"C:\todo\photo-1.jpg".to_string(),
            attempted_destination_path: destination_path.to_string_lossy().to_string(),
            move_strategy: MoveStrategy::Rename,
            result_status: MoveResultStatus::Moved,
            error: None,
        };

        let metadata = MetadataResult {
            file_id: "photo-1".to_string(),
            chosen_date: Some("2026-05-24T01:02:03Z".to_string()),
            chosen_date_source: Some("DateTimeOriginal".to_string()),
            date_kind: None,
            raw_metadata_date: None,
            device_name: None,
            metadata_status: MetadataStatus::Ready,
            error: None,
        };
        let plan = MovePlan {
            file_id: "photo-1".to_string(),
            original_name: "IMG_0001".to_string(),
            destination_dir: r"C:\todo\Processed\2026\05\24".to_string(),
            destination_filename: "IMG_0001.jpg".to_string(),
            destination_path: destination_path.to_string_lossy().to_string(),
            conflict_status: crate::models::ConflictStatus::None,
            plan_status: PlanStatus::Ready,
        };
        let file = FileRecord {
            id: "photo-1".to_string(),
            source_path: r"C:\todo\photo-1.jpg".to_string(),
            relative_path: "photo-1.jpg".to_string(),
            extension: "jpg".to_string(),
            original_name: "fallback_name".to_string(),
            kind: FileKind::Photo,
            scan_status: ScanStatus::Discovered,
        };

        let metadata_by_id = HashMap::from([("photo-1", &metadata)]);
        let plans_by_id = HashMap::from([("photo-1", &plan)]);
        let files_by_id = HashMap::from([("photo-1", &file)]);
        let expected_source_sizes = HashMap::from([("photo-1", Some(4096_u64))]);

        let operation = build_history_operation_for_result(
            "run-1",
            &result,
            &metadata_by_id,
            &plans_by_id,
            &files_by_id,
            &expected_source_sizes,
        );

        assert_eq!(operation.run_id, "run-1");
        assert!(matches!(
            operation.operation_kind,
            HistoryOperationKind::Process
        ));
        assert_eq!(operation.file_id, "photo-1");
        assert_eq!(operation.source_path, r"C:\todo\photo-1.jpg");
        assert_eq!(
            operation.destination_path,
            destination_path.to_string_lossy()
        );
        assert_eq!(operation.expected_source_size, Some(4096));
        assert_eq!(operation.expected_destination_size, Some(4));
        assert_eq!(
            operation.metadata_date.as_deref(),
            Some("2026-05-24T01:02:03Z")
        );
        assert_eq!(operation.original_name, "IMG_0001");
        assert!(matches!(operation.move_strategy, MoveStrategy::Rename));
        assert!(matches!(
            operation.status,
            HistoryOperationStatus::Succeeded
        ));
        assert!(operation.error.is_none());
        assert!(!operation.timestamp.is_empty());
    }

    #[test]
    fn maps_move_result_statuses_to_history_operation_statuses() {
        assert!(matches!(
            history_operation_status_for_move_result(&MoveResultStatus::Moved),
            HistoryOperationStatus::Succeeded
        ));
        assert!(matches!(
            history_operation_status_for_move_result(&MoveResultStatus::Copied),
            HistoryOperationStatus::Succeeded
        ));
        assert!(matches!(
            history_operation_status_for_move_result(&MoveResultStatus::Skipped),
            HistoryOperationStatus::Skipped
        ));
        assert!(matches!(
            history_operation_status_for_move_result(&MoveResultStatus::Failed),
            HistoryOperationStatus::Failed
        ));
    }

    #[test]
    fn chooses_history_run_status_from_error_state() {
        assert!(matches!(
            history_run_status_for_completion(0, false),
            HistoryRunStatus::Completed
        ));
        assert!(matches!(
            history_run_status_for_completion(1, false),
            HistoryRunStatus::CompletedWithErrors
        ));
        assert!(matches!(
            history_run_status_for_completion(0, true),
            HistoryRunStatus::Corrupt
        ));
    }

    #[test]
    fn undoes_rename_operation_when_validation_passes() {
        let temp = tempdir().expect("temp dir");
        let original_source = temp.path().join("todo").join("photo.jpg");
        let processed_destination = temp
            .path()
            .join("todo")
            .join("Processed")
            .join("2026")
            .join("05")
            .join("24")
            .join("photo.jpg");
        fs::create_dir_all(
            processed_destination
                .parent()
                .expect("destination should have parent"),
        )
        .expect("create destination parent");
        fs::write(&processed_destination, b"image-bytes").expect("write processed destination");

        let process_operation = HistoryOperation {
            operation_id: "run-1-op-1".to_string(),
            run_id: "run-1".to_string(),
            file_id: "file-1".to_string(),
            operation_kind: HistoryOperationKind::Process,
            source_path: original_source.to_string_lossy().to_string(),
            destination_path: processed_destination.to_string_lossy().to_string(),
            expected_source_size: Some(11),
            expected_destination_size: Some(11),
            metadata_date: Some("2026-05-24T01:02:03Z".to_string()),
            original_name: "photo".to_string(),
            move_strategy: MoveStrategy::Rename,
            status: HistoryOperationStatus::Succeeded,
            error: None,
            timestamp: "2026-05-24T01:03:03Z".to_string(),
        };

        let result = execute_undo_operation(&process_operation);
        let undo_operation = build_undo_history_operation("run-1", &process_operation, &result);

        assert!(matches!(result.status, HistoryOperationStatus::Succeeded));
        assert!(original_source.exists());
        assert!(!processed_destination.exists());
        assert!(matches!(
            undo_operation.operation_kind,
            HistoryOperationKind::Undo
        ));
        assert_eq!(
            undo_operation.source_path,
            process_operation.destination_path
        );
        assert_eq!(
            undo_operation.destination_path,
            process_operation.source_path
        );
    }

    #[test]
    fn blocks_undo_when_rename_source_already_exists() {
        let temp = tempdir().expect("temp dir");
        let original_source = temp.path().join("todo").join("photo.jpg");
        let processed_destination = temp
            .path()
            .join("todo")
            .join("Processed")
            .join("2026")
            .join("05")
            .join("24")
            .join("photo.jpg");
        fs::create_dir_all(
            processed_destination
                .parent()
                .expect("destination should have parent"),
        )
        .expect("create destination parent");
        fs::create_dir_all(original_source.parent().expect("source should have parent"))
            .expect("create source parent");
        fs::write(&processed_destination, b"image-bytes").expect("write processed destination");
        fs::write(&original_source, b"existing-source").expect("write existing source");

        let process_operation = HistoryOperation {
            operation_id: "run-1-op-1".to_string(),
            run_id: "run-1".to_string(),
            file_id: "file-1".to_string(),
            operation_kind: HistoryOperationKind::Process,
            source_path: original_source.to_string_lossy().to_string(),
            destination_path: processed_destination.to_string_lossy().to_string(),
            expected_source_size: Some(11),
            expected_destination_size: Some(11),
            metadata_date: Some("2026-05-24T01:02:03Z".to_string()),
            original_name: "photo".to_string(),
            move_strategy: MoveStrategy::Rename,
            status: HistoryOperationStatus::Succeeded,
            error: None,
            timestamp: "2026-05-24T01:03:03Z".to_string(),
        };

        let result = execute_undo_operation(&process_operation);
        let undo_operation = build_undo_history_operation("run-1", &process_operation, &result);

        assert!(matches!(result.status, HistoryOperationStatus::Blocked));
        assert!(undo_operation
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("already exists"));
        assert!(original_source.exists());
        assert!(processed_destination.exists());
    }

    #[test]
    fn redoes_rename_operation_when_validation_passes() {
        let temp = tempdir().expect("temp dir");
        let original_source = temp.path().join("todo").join("photo.jpg");
        let processed_destination = temp
            .path()
            .join("todo")
            .join("Processed")
            .join("2026")
            .join("05")
            .join("24")
            .join("photo.jpg");
        fs::create_dir_all(original_source.parent().expect("source should have parent"))
            .expect("create source parent");
        fs::write(&original_source, b"image-bytes").expect("write original source");

        let undo_operation = HistoryOperation {
            operation_id: "run-1-op-2".to_string(),
            run_id: "run-1".to_string(),
            file_id: "file-1".to_string(),
            operation_kind: HistoryOperationKind::Undo,
            source_path: processed_destination.to_string_lossy().to_string(),
            destination_path: original_source.to_string_lossy().to_string(),
            expected_source_size: Some(11),
            expected_destination_size: Some(11),
            metadata_date: Some("2026-05-24T01:02:03Z".to_string()),
            original_name: "photo".to_string(),
            move_strategy: MoveStrategy::Rename,
            status: HistoryOperationStatus::Succeeded,
            error: None,
            timestamp: "2026-05-24T01:08:03Z".to_string(),
        };

        let result = execute_redo_operation(&undo_operation);
        let redo_operation = build_redo_history_operation("run-1", &undo_operation, &result);

        assert!(matches!(result.status, HistoryOperationStatus::Succeeded));
        assert!(!original_source.exists());
        assert!(processed_destination.exists());
        assert!(matches!(
            redo_operation.operation_kind,
            HistoryOperationKind::Redo
        ));
        assert_eq!(redo_operation.source_path, undo_operation.destination_path);
        assert_eq!(redo_operation.destination_path, undo_operation.source_path);
    }

    #[test]
    fn redoes_rename_operation_from_process_record_for_legacy_history() {
        let temp = tempdir().expect("temp dir");
        let original_source = temp.path().join("todo").join("photo.jpg");
        let processed_destination = temp
            .path()
            .join("todo")
            .join("Processed")
            .join("2026")
            .join("05")
            .join("24")
            .join("photo.jpg");
        fs::create_dir_all(original_source.parent().expect("source should have parent"))
            .expect("create source parent");
        fs::write(&original_source, b"image-bytes").expect("write original source");

        let process_operation = HistoryOperation {
            operation_id: "run-1-op-1".to_string(),
            run_id: "run-1".to_string(),
            file_id: "file-1".to_string(),
            operation_kind: HistoryOperationKind::Process,
            source_path: original_source.to_string_lossy().to_string(),
            destination_path: processed_destination.to_string_lossy().to_string(),
            expected_source_size: Some(11),
            expected_destination_size: Some(11),
            metadata_date: Some("2026-05-24T01:02:03Z".to_string()),
            original_name: "photo".to_string(),
            move_strategy: MoveStrategy::Rename,
            status: HistoryOperationStatus::Succeeded,
            error: None,
            timestamp: "2026-05-24T01:03:03Z".to_string(),
        };

        let result = execute_redo_operation(&process_operation);
        let redo_operation = build_redo_history_operation("run-1", &process_operation, &result);

        assert!(matches!(result.status, HistoryOperationStatus::Succeeded));
        assert!(!original_source.exists());
        assert!(processed_destination.exists());
        assert_eq!(redo_operation.source_path, process_operation.source_path);
        assert_eq!(
            redo_operation.destination_path,
            process_operation.destination_path
        );
    }

    #[test]
    fn blocks_redo_when_recorded_destination_already_exists() {
        let temp = tempdir().expect("temp dir");
        let original_source = temp.path().join("todo").join("photo.jpg");
        let processed_destination = temp
            .path()
            .join("todo")
            .join("Processed")
            .join("2026")
            .join("05")
            .join("24")
            .join("photo.jpg");
        fs::create_dir_all(original_source.parent().expect("source should have parent"))
            .expect("create source parent");
        fs::create_dir_all(
            processed_destination
                .parent()
                .expect("destination should have parent"),
        )
        .expect("create destination parent");
        fs::write(&original_source, b"image-bytes").expect("write original source");
        fs::write(&processed_destination, b"existing-destination")
            .expect("write existing destination");

        let undo_operation = HistoryOperation {
            operation_id: "run-1-op-2".to_string(),
            run_id: "run-1".to_string(),
            file_id: "file-1".to_string(),
            operation_kind: HistoryOperationKind::Undo,
            source_path: processed_destination.to_string_lossy().to_string(),
            destination_path: original_source.to_string_lossy().to_string(),
            expected_source_size: Some(11),
            expected_destination_size: Some(11),
            metadata_date: Some("2026-05-24T01:02:03Z".to_string()),
            original_name: "photo".to_string(),
            move_strategy: MoveStrategy::Rename,
            status: HistoryOperationStatus::Succeeded,
            error: None,
            timestamp: "2026-05-24T01:08:03Z".to_string(),
        };

        let result = execute_redo_operation(&undo_operation);
        let redo_operation = build_redo_history_operation("run-1", &undo_operation, &result);

        assert!(matches!(result.status, HistoryOperationStatus::Blocked));
        assert!(redo_operation
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("already exists"));
        assert!(original_source.exists());
        assert!(processed_destination.exists());
    }

    #[cfg(target_os = "windows")]
    mod windows {
        use std::path::Path;

        use crate::commands::strip_windows_verbatim_prefix;

        #[test]
        fn strips_verbatim_disk_prefix_for_explorer() {
            assert_eq!(
                strip_windows_verbatim_prefix(Path::new(r"\\?\C:\todo\IMG 1001.jpg")),
                Path::new(r"C:\todo\IMG 1001.jpg")
            );
        }

        #[test]
        fn strips_verbatim_unc_prefix_for_explorer() {
            assert_eq!(
                strip_windows_verbatim_prefix(Path::new(r"\\?\UNC\nas\photos\TODO\IMG 1001.jpg")),
                Path::new(r"\\nas\photos\TODO\IMG 1001.jpg")
            );
        }
    }
}
