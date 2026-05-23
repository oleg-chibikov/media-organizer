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

use chrono::Local;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::{
    events, logging, metadata,
    models::{
        FileKind, FileRecord, MetadataReadRequest, MetadataResult, MetadataStatus,
        MoveConfirmationSummary, MoveFileEvent, MoveFinishedEvent, MovePlan, MoveResult,
        MoveResultStatus, ProcessingConfig,
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
        let safety_checked = mover::validate_move_safety(&selected_folder, prepared.operations);
        let rechecked = mover::recheck_destination_conflicts(safety_checked.ready);
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
        let mut copied = 0u64;
        let mut skipped = 0u64;
        let mut failed = 0u64;
        let mut moved_source_directories = Vec::new();
        let mut log_file = Some(log_file);

        for result in prepared
            .skipped
            .into_iter()
            .chain(safety_checked.skipped)
            .chain(rechecked.skipped)
        {
            append_move_result_to_log(
                &mut log_file,
                &result,
                &metadata_by_id,
                &plans_by_id,
                &files_by_id,
            );
            count_move_result(&result, &mut moved, &mut copied, &mut skipped, &mut failed);
            let _ = events::emit_move_file(&app, MoveFileEvent { result });
        }

        for operation in rechecked.ready {
            let source_parent = Path::new(&operation.source_path)
                .parent()
                .map(Path::to_path_buf);
            let result = mover::execute_move_with_rename(&operation);
            if matches!(result.result_status, MoveResultStatus::Moved) {
                if let Some(parent) = source_parent {
                    moved_source_directories.push(parent);
                }
            }
            append_move_result_to_log(
                &mut log_file,
                &result,
                &metadata_by_id,
                &plans_by_id,
                &files_by_id,
            );
            count_move_result(&result, &mut moved, &mut copied, &mut skipped, &mut failed);
            let _ = events::emit_move_file(&app, MoveFileEvent { result });
        }

        cleanup_empty_source_directories(&selected_folder, &moved_source_directories);

        let _ = events::emit_move_finished(
            &app,
            MoveFinishedEvent {
                moved,
                copied,
                skipped,
                failed,
            },
        );
    });

    Ok(())
}

#[tauri::command]
pub fn reveal_file_in_explorer(source_path: String) -> Result<(), String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err("File no longer exists on disk.".to_string());
    }

    reveal_path_in_system_explorer(&source)
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

fn cleanup_empty_source_directories(selected_root: &Path, source_directories: &[PathBuf]) {
    let mut visited = HashSet::new();

    for directory in source_directories {
        let key = normalized_path_string(directory);
        if visited.insert(key) {
            prune_empty_directory_chain(selected_root, directory);
        }
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
    use std::fs;

    use tempfile::tempdir;

    use super::cleanup_empty_source_directories;

    #[test]
    fn removes_empty_source_directory_chain_after_moves() {
        let temp = tempdir().expect("temp dir");
        let selected_root = temp.path();
        let nested = selected_root.join("a").join("b");
        fs::create_dir_all(&nested).expect("create nested source dirs");

        cleanup_empty_source_directories(selected_root, &[nested.clone()]);

        assert!(!nested.exists());
        assert!(!selected_root.join("a").exists());
        assert!(selected_root.exists());
    }

    #[test]
    fn stops_pruning_when_parent_contains_files() {
        let temp = tempdir().expect("temp dir");
        let selected_root = temp.path();
        let parent = selected_root.join("a");
        let nested = parent.join("b");
        fs::create_dir_all(&nested).expect("create nested source dirs");
        fs::write(parent.join("keep.txt"), b"keep").expect("write retained file");

        cleanup_empty_source_directories(selected_root, &[nested.clone()]);

        assert!(!nested.exists());
        assert!(parent.exists());
    }

    #[test]
    fn never_removes_selected_root() {
        let temp = tempdir().expect("temp dir");
        let selected_root = temp.path();

        cleanup_empty_source_directories(selected_root, &[selected_root.to_path_buf()]);

        assert!(selected_root.exists());
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
