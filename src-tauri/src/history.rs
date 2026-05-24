use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Manager};

use crate::models::{
    EffectiveDestinationRoot, HistoryIndexEntry, HistoryOperation, HistoryOperationKind,
    HistoryOperationStatus, HistoryRecord, HistoryRun, HistoryRunCompletedRecord,
    HistoryRunDetails, HistoryRunStatus, MoveStrategy,
};

pub const HISTORY_VERSION: u32 = 1;
pub const HISTORY_DIR_NAME: &str = "history";
pub const HISTORY_RUNS_DIR_NAME: &str = "runs";
pub const HISTORY_INDEX_FILENAME: &str = "index.json";
pub const MAX_INDEX_RUNS: usize = 200;

static RUN_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static OPERATION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct ActiveHistoryRun {
    pub run: HistoryRun,
    history_root: PathBuf,
    log_file: File,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct HistoryIndexFile {
    history_version: u32,
    runs: Vec<HistoryIndexEntry>,
}

#[derive(Debug)]
struct RecoveredHistoryRun {
    run: HistoryRun,
    operations: Vec<HistoryOperation>,
    unsafe_for_undo_redo: bool,
    recovery_error: Option<String>,
}

pub fn history_root_for_app(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve appData directory: {error}"))?;
    Ok(history_root_from_app_data(&app_data_dir))
}

pub fn history_root_from_app_data(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(HISTORY_DIR_NAME)
}

pub fn history_runs_dir(history_root: &Path) -> PathBuf {
    history_root.join(HISTORY_RUNS_DIR_NAME)
}

pub fn history_index_path(history_root: &Path) -> PathBuf {
    history_root.join(HISTORY_INDEX_FILENAME)
}

pub fn run_log_filename(started_at: DateTime<Utc>, run_id: &str) -> String {
    format!(
        "{}_{}.jsonl",
        started_at.format("%Y-%m-%dT%H-%M-%SZ"),
        run_id
    )
}

#[allow(dead_code)]
pub fn start_history_run(
    app: &AppHandle,
    selected_folder: &Path,
) -> Result<ActiveHistoryRun, String> {
    start_history_run_with_destination_root(app, selected_folder, None)
}

pub fn start_history_run_with_destination_root(
    app: &AppHandle,
    selected_folder: &Path,
    destination_root: Option<&EffectiveDestinationRoot>,
) -> Result<ActiveHistoryRun, String> {
    let history_root = history_root_for_app(app)?;
    let runs_dir = history_runs_dir(&history_root);
    fs::create_dir_all(&runs_dir).map_err(|error| {
        format!(
            "Failed to create history runs directory at '{}': {error}",
            runs_dir.display()
        )
    })?;

    let started_at = Utc::now();
    let run_id = generate_run_id(started_at);
    let file_name = run_log_filename(started_at, &run_id);
    let log_path = runs_dir.join(file_name);
    let history_file_path = log_path.to_string_lossy().to_string();

    let mut log_file = OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(&log_path)
        .map_err(|error| {
            format!(
                "Failed to open history run log at '{}': {error}",
                log_path.display()
            )
        })?;

    let run = HistoryRun {
        run_id: run_id.clone(),
        history_version: HISTORY_VERSION,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        selected_folder: selected_folder.to_string_lossy().to_string(),
        folder_label: folder_label_from_path(selected_folder),
        destination_root_mode: destination_root.map(|root| root.mode.clone()),
        destination_root_path: destination_root.map(|root| root.root_path.clone()),
        started_at: started_at.to_rfc3339(),
        completed_at: None,
        run_status: HistoryRunStatus::Running,
        history_file_path: history_file_path.clone(),
    };

    append_history_record(
        &mut log_file,
        &HistoryRecord::RunStarted { run: run.clone() },
    )?;

    upsert_history_index_entry(&history_root, index_entry_from_run(&run))?;

    Ok(ActiveHistoryRun {
        run,
        history_root,
        log_file,
    })
}

pub fn complete_history_run(
    active_run: &mut ActiveHistoryRun,
    run_status: HistoryRunStatus,
) -> Result<(), String> {
    let completed_at = Utc::now().to_rfc3339();
    append_history_record(
        &mut active_run.log_file,
        &HistoryRecord::RunCompleted(HistoryRunCompletedRecord {
            run_id: active_run.run.run_id.clone(),
            completed_at: completed_at.clone(),
            run_status: run_status.clone(),
        }),
    )?;

    active_run.run.completed_at = Some(completed_at);
    active_run.run.run_status = run_status;
    upsert_history_index_entry(
        &active_run.history_root,
        index_entry_from_run(&active_run.run),
    )
}

pub fn next_operation_id(run_id: &str) -> String {
    let sequence = OPERATION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{run_id}-{:016x}", sequence)
}

pub fn append_history_operation(
    active_run: &mut ActiveHistoryRun,
    operation: HistoryOperation,
) -> Result<(), String> {
    append_history_record(
        &mut active_run.log_file,
        &HistoryRecord::Operation(operation),
    )
}

pub fn append_history_operation_for_run(
    app: &AppHandle,
    run_id: &str,
    operation: HistoryOperation,
) -> Result<(), String> {
    let history_root = history_root_for_app(app)?;
    let index = load_history_index(&history_root)?;
    let entry = index
        .runs
        .iter()
        .find(|entry| entry.run_id == run_id)
        .ok_or_else(|| format!("History run '{}' was not found in index.", run_id))?;

    let log_path = PathBuf::from(&entry.history_file_path);
    let mut log_file = OpenOptions::new()
        .append(true)
        .open(&log_path)
        .map_err(|error| {
            format!(
                "Failed to open history run log at '{}': {error}",
                log_path.display()
            )
        })?;

    append_history_record(&mut log_file, &HistoryRecord::Operation(operation))
}

pub fn update_history_run_status(
    app: &AppHandle,
    run_id: &str,
    run_status: HistoryRunStatus,
) -> Result<(), String> {
    let history_root = history_root_for_app(app)?;
    let mut index = load_history_index(&history_root)?;
    let entry = index
        .runs
        .iter()
        .find(|entry| entry.run_id == run_id)
        .ok_or_else(|| format!("History run '{}' was not found in index.", run_id))?
        .clone();

    let completed_at = Utc::now().to_rfc3339();
    let log_path = PathBuf::from(&entry.history_file_path);
    let mut log_file = OpenOptions::new()
        .append(true)
        .open(&log_path)
        .map_err(|error| {
            format!(
                "Failed to open history run log at '{}': {error}",
                log_path.display()
            )
        })?;

    append_history_record(
        &mut log_file,
        &HistoryRecord::RunCompleted(HistoryRunCompletedRecord {
            run_id: run_id.to_string(),
            completed_at: completed_at.clone(),
            run_status: run_status.clone(),
        }),
    )?;

    let Some(index_entry) = index.runs.iter_mut().find(|entry| entry.run_id == run_id) else {
        return Err(format!("History run '{}' was not found in index.", run_id));
    };
    index_entry.completed_at = Some(completed_at);
    index_entry.run_status = run_status;
    write_history_index(&history_root, &index)
}

pub fn list_history_runs(app: &AppHandle) -> Result<Vec<HistoryIndexEntry>, String> {
    let history_root = history_root_for_app(app)?;
    list_history_runs_from_root(&history_root)
}

pub fn get_history_run(app: &AppHandle, run_id: &str) -> Result<Option<HistoryRunDetails>, String> {
    let history_root = history_root_for_app(app)?;
    get_history_run_from_root(&history_root, run_id)
}

fn list_history_runs_from_root(history_root: &Path) -> Result<Vec<HistoryIndexEntry>, String> {
    let mut index = load_history_index(history_root)?;
    let mut did_change = false;
    let mut recovered_entries = Vec::with_capacity(index.runs.len());

    for entry in &index.runs {
        let recovered = recover_history_run_from_entry(entry);
        let mut next_entry = entry.clone();
        next_entry.completed_at = recovered.run.completed_at.clone();
        next_entry.run_status = recovered.run.run_status.clone();
        next_entry.undo_available =
            !recovered.unsafe_for_undo_redo && undo_candidate_exists(&recovered);
        next_entry.redo_available =
            !recovered.unsafe_for_undo_redo && redo_candidate_exists(&recovered);
        if next_entry != *entry {
            did_change = true;
        }
        recovered_entries.push(next_entry);
    }

    recovered_entries.sort_by(|left, right| {
        right
            .started_at
            .cmp(&left.started_at)
            .then_with(|| right.run_id.cmp(&left.run_id))
    });
    recovered_entries.truncate(MAX_INDEX_RUNS);

    if recovered_entries != index.runs {
        did_change = true;
    }

    if did_change {
        index.runs = recovered_entries.clone();
        write_history_index(history_root, &index)?;
    }

    Ok(recovered_entries)
}

fn get_history_run_from_root(
    history_root: &Path,
    run_id: &str,
) -> Result<Option<HistoryRunDetails>, String> {
    let index = load_history_index(history_root)?;
    let Some(entry) = index.runs.iter().find(|entry| entry.run_id == run_id) else {
        return Ok(None);
    };

    let recovered = recover_history_run_from_entry(entry);
    Ok(Some(HistoryRunDetails {
        run: recovered.run,
        operations: recovered
            .operations
            .into_iter()
            .filter(|op| matches!(op.operation_kind, HistoryOperationKind::Process))
            .collect(),
        unsafe_for_undo_redo: recovered.unsafe_for_undo_redo,
        recovery_error: recovered.recovery_error,
    }))
}

fn undo_candidate_exists(recovered: &RecoveredHistoryRun) -> bool {
    matches!(
        recovered.run.run_status,
        HistoryRunStatus::Completed
            | HistoryRunStatus::CompletedWithErrors
            | HistoryRunStatus::Redone
    ) && !collect_undoable_operations(&recovered.operations).is_empty()
}

fn redo_candidate_exists(recovered: &RecoveredHistoryRun) -> bool {
    matches!(
        recovered.run.run_status,
        HistoryRunStatus::Undone | HistoryRunStatus::PartiallyUndone
    ) && !collect_redoable_operations(&recovered.operations).is_empty()
}

pub fn collect_undoable_operations(operations: &[HistoryOperation]) -> Vec<HistoryOperation> {
    let has_undo_redo_ops = operations.iter().any(|op| {
        matches!(
            op.operation_kind,
            HistoryOperationKind::Undo | HistoryOperationKind::Redo
        )
    });

    let mut undoable: Vec<HistoryOperation> = if has_undo_redo_ops {
        let mut last_succeeded_by_file =
            std::collections::HashMap::<String, &HistoryOperation>::new();
        for operation in operations {
            if !matches!(operation.status, HistoryOperationStatus::Succeeded) {
                continue;
            }
            last_succeeded_by_file.insert(operation.file_id.clone(), operation);
        }
        last_succeeded_by_file
            .values()
            .filter_map(|operation| {
                if matches!(
                    operation.operation_kind,
                    HistoryOperationKind::Process | HistoryOperationKind::Redo
                ) && matches!(
                    operation.move_strategy,
                    MoveStrategy::Rename | MoveStrategy::CopyKeepSource
                ) {
                    return Some((*operation).clone());
                }
                None
            })
            .collect()
    } else {
        operations
            .iter()
            .filter(|op| {
                matches!(op.status, HistoryOperationStatus::Succeeded)
                    && matches!(op.operation_kind, HistoryOperationKind::Process)
                    && matches!(
                        op.move_strategy,
                        MoveStrategy::Rename | MoveStrategy::CopyKeepSource
                    )
            })
            .cloned()
            .collect()
    };

    undoable.sort_by(|left, right| left.operation_id.cmp(&right.operation_id));
    undoable
}

pub fn collect_redoable_operations(operations: &[HistoryOperation]) -> Vec<HistoryOperation> {
    let has_undo_ops = operations
        .iter()
        .any(|op| matches!(op.operation_kind, HistoryOperationKind::Undo));

    let mut redoable: Vec<HistoryOperation> = if has_undo_ops {
        let mut last_succeeded_by_file =
            std::collections::HashMap::<String, &HistoryOperation>::new();
        for operation in operations {
            if !matches!(operation.status, HistoryOperationStatus::Succeeded) {
                continue;
            }
            last_succeeded_by_file.insert(operation.file_id.clone(), operation);
        }
        last_succeeded_by_file
            .values()
            .filter_map(|operation| {
                if matches!(operation.operation_kind, HistoryOperationKind::Undo)
                    && matches!(
                        operation.move_strategy,
                        MoveStrategy::Rename | MoveStrategy::CopyKeepSource
                    )
                {
                    return Some((*operation).clone());
                }
                None
            })
            .collect()
    } else {
        operations
            .iter()
            .filter(|op| {
                matches!(op.status, HistoryOperationStatus::Succeeded)
                    && matches!(op.operation_kind, HistoryOperationKind::Process)
                    && matches!(
                        op.move_strategy,
                        MoveStrategy::Rename | MoveStrategy::CopyKeepSource
                    )
            })
            .cloned()
            .collect()
    };

    redoable.sort_by(|left, right| left.operation_id.cmp(&right.operation_id));
    redoable
}

fn recover_history_run_from_entry(entry: &HistoryIndexEntry) -> RecoveredHistoryRun {
    let history_path = PathBuf::from(&entry.history_file_path);
    let parse_result = parse_run_log_records(&history_path, &entry.run_id);
    let mut run = parse_result
        .started_run
        .unwrap_or_else(|| run_from_index_entry(entry));
    run.history_file_path = entry.history_file_path.clone();

    let mut unsafe_for_undo_redo = false;
    let mut recovery_error = parse_result.recovery_error;

    if parse_result.corrupt {
        run.run_status = HistoryRunStatus::Corrupt;
        unsafe_for_undo_redo = true;
    } else if parse_result.partial || parse_result.incomplete {
        run.run_status = HistoryRunStatus::Partial;
        unsafe_for_undo_redo = true;
    } else if let Some(completed) = parse_result.completed_record {
        run.completed_at = Some(completed.completed_at);
        run.run_status = completed.run_status;
    }

    if recovery_error.is_some() {
        unsafe_for_undo_redo = true;
    }

    RecoveredHistoryRun {
        run,
        operations: parse_result.operations,
        unsafe_for_undo_redo,
        recovery_error: recovery_error.take(),
    }
}

fn run_from_index_entry(entry: &HistoryIndexEntry) -> HistoryRun {
    HistoryRun {
        run_id: entry.run_id.clone(),
        history_version: HISTORY_VERSION,
        app_version: String::new(),
        selected_folder: entry.selected_folder.clone(),
        folder_label: entry.folder_label.clone(),
        destination_root_mode: None,
        destination_root_path: None,
        started_at: entry.started_at.clone(),
        completed_at: entry.completed_at.clone(),
        run_status: entry.run_status.clone(),
        history_file_path: entry.history_file_path.clone(),
    }
}

struct ParsedRunLog {
    started_run: Option<HistoryRun>,
    operations: Vec<HistoryOperation>,
    completed_record: Option<HistoryRunCompletedRecord>,
    partial: bool,
    incomplete: bool,
    corrupt: bool,
    recovery_error: Option<String>,
}

fn parse_run_log_records(history_file_path: &Path, expected_run_id: &str) -> ParsedRunLog {
    let mut parsed = ParsedRunLog {
        started_run: None,
        operations: Vec::new(),
        completed_record: None,
        partial: false,
        incomplete: false,
        corrupt: false,
        recovery_error: None,
    };

    let file = match File::open(history_file_path) {
        Ok(file) => file,
        Err(error) => {
            parsed.corrupt = true;
            parsed.recovery_error = Some(format!(
                "Failed to open history run log at '{}': {error}",
                history_file_path.display()
            ));
            return parsed;
        }
    };

    let mut reader = BufReader::new(file);
    let mut line_bytes = Vec::new();
    let mut line_number = 0usize;

    loop {
        line_bytes.clear();
        match reader.read_until(b'\n', &mut line_bytes) {
            Ok(0) => break,
            Ok(_) => {}
            Err(error) => {
                parsed.corrupt = true;
                parsed.recovery_error = Some(format!(
                    "Failed reading history run log at '{}': {error}",
                    history_file_path.display()
                ));
                return parsed;
            }
        }

        line_number += 1;
        let has_newline = line_bytes.last() == Some(&b'\n');
        if has_newline {
            line_bytes.pop();
        }
        if line_bytes.last() == Some(&b'\r') {
            line_bytes.pop();
        }
        if line_bytes.is_empty() {
            continue;
        }

        let line = match std::str::from_utf8(&line_bytes) {
            Ok(value) => value,
            Err(error) => {
                if !has_newline {
                    parsed.partial = true;
                    break;
                }
                parsed.corrupt = true;
                parsed.recovery_error = Some(format!(
                    "Malformed UTF-8 at line {line_number} in '{}': {error}",
                    history_file_path.display()
                ));
                break;
            }
        };

        let record: HistoryRecord = match serde_json::from_str(line) {
            Ok(record) => record,
            Err(error) => {
                if !has_newline {
                    parsed.partial = true;
                    break;
                }
                parsed.corrupt = true;
                parsed.recovery_error = Some(format!(
                    "Malformed history record at line {line_number} in '{}': {error}",
                    history_file_path.display()
                ));
                break;
            }
        };

        match record {
            HistoryRecord::RunStarted { run } => {
                if run.run_id != expected_run_id {
                    parsed.corrupt = true;
                    parsed.recovery_error = Some(format!(
                        "Run id mismatch at line {line_number} in '{}': expected '{}', found '{}'",
                        history_file_path.display(),
                        expected_run_id,
                        run.run_id
                    ));
                    break;
                }
                parsed.started_run = Some(run);
            }
            HistoryRecord::Operation(operation) => {
                if operation.run_id != expected_run_id {
                    parsed.corrupt = true;
                    parsed.recovery_error = Some(format!(
                        "Operation run id mismatch at line {line_number} in '{}': expected '{}', found '{}'",
                        history_file_path.display(),
                        expected_run_id,
                        operation.run_id
                    ));
                    break;
                }
                parsed.operations.push(operation);
            }
            HistoryRecord::RunCompleted(record) => {
                if record.run_id != expected_run_id {
                    parsed.corrupt = true;
                    parsed.recovery_error = Some(format!(
                        "Completion run id mismatch at line {line_number} in '{}': expected '{}', found '{}'",
                        history_file_path.display(),
                        expected_run_id,
                        record.run_id
                    ));
                    break;
                }
                parsed.completed_record = Some(record);
            }
        }
    }

    if parsed.started_run.is_none() {
        parsed.incomplete = true;
    } else if parsed.completed_record.is_none() {
        parsed.incomplete = true;
    }

    parsed
}

fn generate_run_id(now: DateTime<Utc>) -> String {
    let sequence = RUN_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "{:x}{:x}{:x}",
        now.timestamp_micros().unsigned_abs(),
        std::process::id(),
        sequence
    )
}

fn folder_label_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|segment| segment.to_str())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn append_history_record(file: &mut File, record: &HistoryRecord) -> Result<(), String> {
    let line = serde_json::to_string(record)
        .map_err(|error| format!("Failed to serialize history record: {error}"))?;
    file.write_all(line.as_bytes())
        .map_err(|error| format!("Failed to append history record: {error}"))?;
    file.write_all(b"\n")
        .map_err(|error| format!("Failed to append history record newline: {error}"))?;
    file.flush()
        .map_err(|error| format!("Failed to flush history record: {error}"))
}

fn load_history_index(history_root: &Path) -> Result<HistoryIndexFile, String> {
    let index_path = history_index_path(history_root);
    if !index_path.exists() {
        return Ok(HistoryIndexFile {
            history_version: HISTORY_VERSION,
            runs: Vec::new(),
        });
    }

    let raw = fs::read_to_string(&index_path).map_err(|error| {
        format!(
            "Failed to read history index at '{}': {error}",
            index_path.display()
        )
    })?;

    serde_json::from_str(&raw).map_err(|error| {
        format!(
            "Failed to parse history index at '{}': {error}",
            index_path.display()
        )
    })
}

fn write_history_index(history_root: &Path, index: &HistoryIndexFile) -> Result<(), String> {
    fs::create_dir_all(history_root).map_err(|error| {
        format!(
            "Failed to create history directory at '{}': {error}",
            history_root.display()
        )
    })?;

    let serialized = serde_json::to_string_pretty(index)
        .map_err(|error| format!("Failed to serialize history index: {error}"))?;

    let index_path = history_index_path(history_root);
    fs::write(&index_path, serialized).map_err(|error| {
        format!(
            "Failed to write history index at '{}': {error}",
            index_path.display()
        )
    })
}

fn upsert_history_index_entry(history_root: &Path, entry: HistoryIndexEntry) -> Result<(), String> {
    let mut index = load_history_index(history_root)?;

    if let Some(existing) = index.runs.iter_mut().find(|run| run.run_id == entry.run_id) {
        *existing = entry;
    } else {
        index.runs.push(entry);
    }

    index.runs.sort_by(|left, right| {
        right
            .started_at
            .cmp(&left.started_at)
            .then_with(|| right.run_id.cmp(&left.run_id))
    });
    index.runs.truncate(MAX_INDEX_RUNS);
    write_history_index(history_root, &index)
}

fn index_entry_from_run(run: &HistoryRun) -> HistoryIndexEntry {
    HistoryIndexEntry {
        run_id: run.run_id.clone(),
        folder_label: run.folder_label.clone(),
        selected_folder: run.selected_folder.clone(),
        started_at: run.started_at.clone(),
        completed_at: run.completed_at.clone(),
        run_status: run.run_status.clone(),
        undo_available: false,
        redo_available: false,
        history_file_path: run.history_file_path.clone(),
    }
}

pub fn delete_history_run(app: &AppHandle, run_id: &str) -> Result<(), String> {
    let history_root = history_root_for_app(app)?;
    let mut index = load_history_index(&history_root)?;

    let entry = index
        .runs
        .iter()
        .find(|entry| entry.run_id == run_id)
        .ok_or_else(|| format!("History run '{run_id}' was not found in index."))?
        .clone();

    index.runs.retain(|entry| entry.run_id != run_id);
    write_history_index(&history_root, &index)?;

    let log_path = PathBuf::from(&entry.history_file_path);
    if log_path.exists() {
        fs::remove_file(&log_path).map_err(|error| {
            format!(
                "Failed to remove history log file at '{}': {error}",
                log_path.display()
            )
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration, TimeZone};
    use tempfile::tempdir;

    #[test]
    fn builds_history_paths_under_app_data() {
        let app_data = Path::new(r"C:\Users\user\AppData\Roaming\media-organizer");
        let history_root = history_root_from_app_data(app_data);
        let runs_dir = history_runs_dir(&history_root);
        let index_path = history_index_path(&history_root);

        assert_eq!(
            history_root,
            Path::new(r"C:\Users\user\AppData\Roaming\media-organizer\history")
        );
        assert_eq!(
            runs_dir,
            Path::new(r"C:\Users\user\AppData\Roaming\media-organizer\history\runs")
        );
        assert_eq!(
            index_path,
            Path::new(r"C:\Users\user\AppData\Roaming\media-organizer\history\index.json")
        );
    }

    #[test]
    fn generates_sortable_run_log_filename() {
        let started_at: DateTime<Utc> = Utc
            .with_ymd_and_hms(2026, 5, 24, 1, 2, 3)
            .single()
            .expect("valid utc time");

        let file_name = run_log_filename(started_at, "runabc123");
        assert_eq!(file_name, "2026-05-24T01-02-03Z_runabc123.jsonl");
    }

    #[test]
    fn loads_legacy_history_run_without_destination_root_fields() {
        let raw = r#"{
            "run_id": "run-legacy",
            "history_version": 1,
            "app_version": "0.1.0",
            "selected_folder": "C:\\todo",
            "folder_label": "todo",
            "started_at": "2026-05-24T01:02:03Z",
            "completed_at": null,
            "run_status": "running",
            "history_file_path": "C:\\appdata\\history\\runs\\run-legacy.jsonl"
        }"#;

        let run: HistoryRun =
            serde_json::from_str(raw).expect("legacy run should deserialize safely");

        assert_eq!(run.destination_root_mode, None);
        assert_eq!(run.destination_root_path, None);
    }

    #[test]
    fn serializes_history_run_destination_root_fields() {
        let run = HistoryRun {
            run_id: "run-custom-root".to_string(),
            history_version: HISTORY_VERSION,
            app_version: "0.1.0".to_string(),
            selected_folder: r"C:\todo".to_string(),
            folder_label: "todo".to_string(),
            destination_root_mode: Some(crate::models::DestinationRootMode::CustomRoot),
            destination_root_path: Some(r"D:\Media".to_string()),
            started_at: "2026-05-24T01:02:03Z".to_string(),
            completed_at: None,
            run_status: HistoryRunStatus::Running,
            history_file_path: r"C:\appdata\history\runs\run-custom-root.jsonl".to_string(),
        };

        let serialized = serde_json::to_string(&run).expect("run should serialize");

        assert!(serialized.contains("\"destination_root_mode\":\"custom_root\""));
        assert!(serialized.contains("\"destination_root_path\":\"D:\\\\Media\""));
    }

    #[test]
    fn appends_jsonl_history_records() {
        let temp = tempdir().expect("temp dir");
        let log_path = temp.path().join("history.jsonl");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("open log");

        let started_record = HistoryRecord::RunStarted {
            run: HistoryRun {
                run_id: "run-1".to_string(),
                history_version: HISTORY_VERSION,
                app_version: "0.1.0".to_string(),
                selected_folder: r"C:\todo".to_string(),
                folder_label: "todo".to_string(),
                destination_root_mode: None,
                destination_root_path: None,
                started_at: "2026-05-24T01:02:03Z".to_string(),
                completed_at: None,
                run_status: HistoryRunStatus::Running,
                history_file_path: r"C:\appdata\history\runs\run-1.jsonl".to_string(),
            },
        };
        let completed_record = HistoryRecord::RunCompleted(HistoryRunCompletedRecord {
            run_id: "run-1".to_string(),
            completed_at: "2026-05-24T01:05:00Z".to_string(),
            run_status: HistoryRunStatus::Completed,
        });
        let operation_record = HistoryRecord::Operation(HistoryOperation {
            operation_id: "run-1-0000000000000001".to_string(),
            run_id: "run-1".to_string(),
            file_id: "photo-1".to_string(),
            operation_kind: crate::models::HistoryOperationKind::Process,
            source_path: r"C:\todo\photo.jpg".to_string(),
            destination_path: r"C:\todo\Processed\2026\05\24\photo.jpg".to_string(),
            expected_source_size: Some(1024),
            expected_destination_size: Some(1024),
            metadata_date: Some("2026-05-24T01:02:03Z".to_string()),
            original_name: "photo".to_string(),
            move_strategy: crate::models::MoveStrategy::Rename,
            status: crate::models::HistoryOperationStatus::Succeeded,
            error: None,
            timestamp: "2026-05-24T01:03:03Z".to_string(),
        });

        append_history_record(&mut file, &started_record).expect("append start");
        append_history_record(&mut file, &operation_record).expect("append operation");
        append_history_record(&mut file, &completed_record).expect("append complete");
        drop(file);

        let contents = fs::read_to_string(&log_path).expect("read log");
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains(r#""record_type":"run_started""#));
        assert!(lines[1].contains(r#""record_type":"operation""#));
        assert!(lines[2].contains(r#""record_type":"run_completed""#));
    }

    #[test]
    fn keeps_index_sorted_and_bounded() {
        let temp = tempdir().expect("temp dir");
        let history_root = temp.path().join("history");
        let base_started_at = Utc
            .with_ymd_and_hms(2026, 5, 24, 0, 0, 0)
            .single()
            .expect("valid utc time");

        for value in 0..(MAX_INDEX_RUNS + 5) {
            let started_at = base_started_at + Duration::seconds(value as i64);
            let entry = HistoryIndexEntry {
                run_id: format!("run-{value}"),
                folder_label: "todo".to_string(),
                selected_folder: r"C:\todo".to_string(),
                started_at: started_at.to_rfc3339(),
                completed_at: None,
                run_status: HistoryRunStatus::Running,
                undo_available: false,
                redo_available: false,
                history_file_path: format!(r"C:\appdata\history\runs\run-{value}.jsonl"),
            };
            upsert_history_index_entry(&history_root, entry).expect("upsert index entry");
        }

        let index_path = history_index_path(&history_root);
        let raw = fs::read_to_string(index_path).expect("read index");
        let index: HistoryIndexFile = serde_json::from_str(&raw).expect("parse index");

        assert_eq!(index.runs.len(), MAX_INDEX_RUNS);
        assert_eq!(index.runs[0].run_id, format!("run-{}", MAX_INDEX_RUNS + 4));
        assert_eq!(index.runs[MAX_INDEX_RUNS - 1].run_id, "run-5");
    }

    #[test]
    fn collects_undoable_operations_for_supported_strategies_only() {
        let operations = vec![
            HistoryOperation {
                operation_id: "run-1-01".to_string(),
                run_id: "run-1".to_string(),
                file_id: "rename-file".to_string(),
                operation_kind: crate::models::HistoryOperationKind::Process,
                source_path: r"C:\todo\rename.jpg".to_string(),
                destination_path: r"C:\todo\Processed\rename.jpg".to_string(),
                expected_source_size: Some(10),
                expected_destination_size: Some(10),
                metadata_date: None,
                original_name: "rename".to_string(),
                move_strategy: crate::models::MoveStrategy::Rename,
                status: crate::models::HistoryOperationStatus::Succeeded,
                error: None,
                timestamp: "2026-05-24T01:03:03Z".to_string(),
            },
            HistoryOperation {
                operation_id: "run-1-02".to_string(),
                run_id: "run-1".to_string(),
                file_id: "copy-file".to_string(),
                operation_kind: crate::models::HistoryOperationKind::Process,
                source_path: r"C:\todo\copy.jpg".to_string(),
                destination_path: r"C:\todo\Processed\copy.jpg".to_string(),
                expected_source_size: Some(11),
                expected_destination_size: Some(11),
                metadata_date: None,
                original_name: "copy".to_string(),
                move_strategy: crate::models::MoveStrategy::CopyKeepSource,
                status: crate::models::HistoryOperationStatus::Succeeded,
                error: None,
                timestamp: "2026-05-24T01:04:03Z".to_string(),
            },
            HistoryOperation {
                operation_id: "run-1-03".to_string(),
                run_id: "run-1".to_string(),
                file_id: "skipped-file".to_string(),
                operation_kind: crate::models::HistoryOperationKind::Process,
                source_path: r"C:\todo\skip.jpg".to_string(),
                destination_path: r"C:\todo\Processed\skip.jpg".to_string(),
                expected_source_size: Some(12),
                expected_destination_size: Some(12),
                metadata_date: None,
                original_name: "skip".to_string(),
                move_strategy: crate::models::MoveStrategy::Rename,
                status: crate::models::HistoryOperationStatus::Skipped,
                error: None,
                timestamp: "2026-05-24T01:05:03Z".to_string(),
            },
        ];

        let undoable = collect_undoable_operations(&operations);
        assert_eq!(undoable.len(), 2);
        assert!(undoable
            .iter()
            .any(|operation| operation.file_id == "rename-file"));
        assert!(undoable
            .iter()
            .any(|operation| operation.file_id == "copy-file"));
    }

    #[test]
    fn excludes_successfully_undone_operations_from_undo_candidates() {
        let operations = vec![
            HistoryOperation {
                operation_id: "run-2-01".to_string(),
                run_id: "run-2".to_string(),
                file_id: "file-1".to_string(),
                operation_kind: crate::models::HistoryOperationKind::Process,
                source_path: r"C:\todo\photo.jpg".to_string(),
                destination_path: r"C:\todo\Processed\photo.jpg".to_string(),
                expected_source_size: Some(10),
                expected_destination_size: Some(10),
                metadata_date: None,
                original_name: "photo".to_string(),
                move_strategy: crate::models::MoveStrategy::Rename,
                status: crate::models::HistoryOperationStatus::Succeeded,
                error: None,
                timestamp: "2026-05-24T01:03:03Z".to_string(),
            },
            HistoryOperation {
                operation_id: "run-2-02".to_string(),
                run_id: "run-2".to_string(),
                file_id: "file-1".to_string(),
                operation_kind: crate::models::HistoryOperationKind::Undo,
                source_path: r"C:\todo\Processed\photo.jpg".to_string(),
                destination_path: r"C:\todo\photo.jpg".to_string(),
                expected_source_size: Some(10),
                expected_destination_size: Some(10),
                metadata_date: None,
                original_name: "photo".to_string(),
                move_strategy: crate::models::MoveStrategy::Rename,
                status: crate::models::HistoryOperationStatus::Succeeded,
                error: None,
                timestamp: "2026-05-24T01:06:03Z".to_string(),
            },
        ];

        let undoable = collect_undoable_operations(&operations);
        assert!(undoable.is_empty());
    }

    #[test]
    fn collects_redoable_operations_when_latest_success_is_undo() {
        let operations = vec![
            HistoryOperation {
                operation_id: "run-3-01".to_string(),
                run_id: "run-3".to_string(),
                file_id: "file-1".to_string(),
                operation_kind: crate::models::HistoryOperationKind::Process,
                source_path: r"C:\todo\photo.jpg".to_string(),
                destination_path: r"C:\todo\Processed\photo.jpg".to_string(),
                expected_source_size: Some(10),
                expected_destination_size: Some(10),
                metadata_date: None,
                original_name: "photo".to_string(),
                move_strategy: crate::models::MoveStrategy::Rename,
                status: crate::models::HistoryOperationStatus::Succeeded,
                error: None,
                timestamp: "2026-05-24T01:03:03Z".to_string(),
            },
            HistoryOperation {
                operation_id: "run-3-02".to_string(),
                run_id: "run-3".to_string(),
                file_id: "file-1".to_string(),
                operation_kind: crate::models::HistoryOperationKind::Undo,
                source_path: r"C:\todo\Processed\photo.jpg".to_string(),
                destination_path: r"C:\todo\photo.jpg".to_string(),
                expected_source_size: Some(10),
                expected_destination_size: Some(10),
                metadata_date: None,
                original_name: "photo".to_string(),
                move_strategy: crate::models::MoveStrategy::Rename,
                status: crate::models::HistoryOperationStatus::Succeeded,
                error: None,
                timestamp: "2026-05-24T01:06:03Z".to_string(),
            },
        ];

        let redoable = collect_redoable_operations(&operations);
        assert_eq!(redoable.len(), 1);
        assert_eq!(redoable[0].file_id, "file-1");
        assert!(matches!(
            redoable[0].operation_kind,
            crate::models::HistoryOperationKind::Undo
        ));
    }

    #[test]
    fn excludes_successfully_redone_operations_from_redo_candidates() {
        let operations = vec![
            HistoryOperation {
                operation_id: "run-4-01".to_string(),
                run_id: "run-4".to_string(),
                file_id: "file-1".to_string(),
                operation_kind: crate::models::HistoryOperationKind::Undo,
                source_path: r"C:\todo\Processed\photo.jpg".to_string(),
                destination_path: r"C:\todo\photo.jpg".to_string(),
                expected_source_size: Some(10),
                expected_destination_size: Some(10),
                metadata_date: None,
                original_name: "photo".to_string(),
                move_strategy: crate::models::MoveStrategy::Rename,
                status: crate::models::HistoryOperationStatus::Succeeded,
                error: None,
                timestamp: "2026-05-24T01:06:03Z".to_string(),
            },
            HistoryOperation {
                operation_id: "run-4-02".to_string(),
                run_id: "run-4".to_string(),
                file_id: "file-1".to_string(),
                operation_kind: crate::models::HistoryOperationKind::Redo,
                source_path: r"C:\todo\photo.jpg".to_string(),
                destination_path: r"C:\todo\Processed\photo.jpg".to_string(),
                expected_source_size: Some(10),
                expected_destination_size: Some(10),
                metadata_date: None,
                original_name: "photo".to_string(),
                move_strategy: crate::models::MoveStrategy::Rename,
                status: crate::models::HistoryOperationStatus::Succeeded,
                error: None,
                timestamp: "2026-05-24T01:07:03Z".to_string(),
            },
        ];

        let redoable = collect_redoable_operations(&operations);
        assert!(redoable.is_empty());
    }

    #[test]
    fn recovers_partial_trailing_history_record_as_partial_and_unsafe() {
        let temp = tempdir().expect("temp dir");
        let history_root = temp.path().join("history");
        let runs_dir = history_runs_dir(&history_root);
        fs::create_dir_all(&runs_dir).expect("create runs dir");
        let log_path = runs_dir.join("2026-05-24T01-02-03Z_run-1.jsonl");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path)
            .expect("open run log");

        let run = HistoryRun {
            run_id: "run-1".to_string(),
            history_version: HISTORY_VERSION,
            app_version: "0.1.0".to_string(),
            selected_folder: r"C:\todo".to_string(),
            folder_label: "todo".to_string(),
            destination_root_mode: None,
            destination_root_path: None,
            started_at: "2026-05-24T01:02:03Z".to_string(),
            completed_at: None,
            run_status: HistoryRunStatus::Running,
            history_file_path: log_path.to_string_lossy().to_string(),
        };
        append_history_record(&mut file, &HistoryRecord::RunStarted { run: run.clone() })
            .expect("write start record");
        append_history_record(
            &mut file,
            &HistoryRecord::Operation(HistoryOperation {
                operation_id: "run-1-0000000000000001".to_string(),
                run_id: "run-1".to_string(),
                file_id: "file-1".to_string(),
                operation_kind: crate::models::HistoryOperationKind::Process,
                source_path: r"C:\todo\in.jpg".to_string(),
                destination_path: r"C:\todo\Processed\out.jpg".to_string(),
                expected_source_size: Some(12),
                expected_destination_size: Some(12),
                metadata_date: Some("2026-05-24T01:00:00Z".to_string()),
                original_name: "in".to_string(),
                move_strategy: crate::models::MoveStrategy::Rename,
                status: crate::models::HistoryOperationStatus::Succeeded,
                error: None,
                timestamp: "2026-05-24T01:03:03Z".to_string(),
            }),
        )
        .expect("write operation record");
        file.write_all(br#"{"record_type":"run_completed""#)
            .expect("write partial trailing record");
        file.flush().expect("flush partial record");

        upsert_history_index_entry(
            &history_root,
            HistoryIndexEntry {
                run_id: "run-1".to_string(),
                folder_label: "todo".to_string(),
                selected_folder: r"C:\todo".to_string(),
                started_at: "2026-05-24T01:02:03Z".to_string(),
                completed_at: None,
                run_status: HistoryRunStatus::Running,
                undo_available: true,
                redo_available: true,
                history_file_path: log_path.to_string_lossy().to_string(),
            },
        )
        .expect("write index entry");

        let runs = list_history_runs_from_root(&history_root).expect("list runs");
        assert_eq!(runs.len(), 1);
        assert!(matches!(runs[0].run_status, HistoryRunStatus::Partial));
        assert!(!runs[0].undo_available);
        assert!(!runs[0].redo_available);

        let details = get_history_run_from_root(&history_root, "run-1")
            .expect("load run details")
            .expect("run details should exist");
        assert!(matches!(details.run.run_status, HistoryRunStatus::Partial));
        assert!(details.unsafe_for_undo_redo);
        assert_eq!(details.operations.len(), 1);
    }

    #[test]
    fn marks_malformed_history_record_as_corrupt_and_unsafe() {
        let temp = tempdir().expect("temp dir");
        let history_root = temp.path().join("history");
        let runs_dir = history_runs_dir(&history_root);
        fs::create_dir_all(&runs_dir).expect("create runs dir");
        let log_path = runs_dir.join("2026-05-24T01-02-03Z_run-2.jsonl");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path)
            .expect("open run log");

        append_history_record(
            &mut file,
            &HistoryRecord::RunStarted {
                run: HistoryRun {
                    run_id: "run-2".to_string(),
                    history_version: HISTORY_VERSION,
                    app_version: "0.1.0".to_string(),
                    selected_folder: r"C:\todo".to_string(),
                    folder_label: "todo".to_string(),
                    destination_root_mode: None,
                    destination_root_path: None,
                    started_at: "2026-05-24T01:02:03Z".to_string(),
                    completed_at: None,
                    run_status: HistoryRunStatus::Running,
                    history_file_path: log_path.to_string_lossy().to_string(),
                },
            },
        )
        .expect("write start record");
        file.write_all(b"not-json\n")
            .expect("write malformed record");
        file.flush().expect("flush malformed record");

        upsert_history_index_entry(
            &history_root,
            HistoryIndexEntry {
                run_id: "run-2".to_string(),
                folder_label: "todo".to_string(),
                selected_folder: r"C:\todo".to_string(),
                started_at: "2026-05-24T01:02:03Z".to_string(),
                completed_at: None,
                run_status: HistoryRunStatus::Running,
                undo_available: true,
                redo_available: true,
                history_file_path: log_path.to_string_lossy().to_string(),
            },
        )
        .expect("write index entry");

        let runs = list_history_runs_from_root(&history_root).expect("list runs");
        assert_eq!(runs.len(), 1);
        assert!(matches!(runs[0].run_status, HistoryRunStatus::Corrupt));
        assert!(!runs[0].undo_available);
        assert!(!runs[0].redo_available);

        let details = get_history_run_from_root(&history_root, "run-2")
            .expect("load run details")
            .expect("run details should exist");
        assert!(matches!(details.run.run_status, HistoryRunStatus::Corrupt));
        assert!(details.unsafe_for_undo_redo);
        assert!(details.recovery_error.is_some());
    }
}
