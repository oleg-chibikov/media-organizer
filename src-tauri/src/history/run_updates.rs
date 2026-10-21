use std::{fs, path::PathBuf};

use chrono::Utc;
use tauri::AppHandle;

use super::{
    index::{load_history_index, write_history_index},
    log_file::{append_history_record, open_run_log_for_append},
    paths::history_root_for_app,
};
use crate::models::{HistoryOperation, HistoryRecord, HistoryRunCompletedRecord, HistoryRunStatus};

/// Appends an undo/redo operation to the log of an already completed run.
pub(crate) fn append_history_operation_for_run(
    app: &AppHandle,
    run_id: &str,
    operation: HistoryOperation,
) -> Result<(), String> {
    let history_root = history_root_for_app(app)?;
    let index = load_history_index(&history_root)?;
    let log_path = PathBuf::from(&index.find(run_id)?.history_file_path);
    let mut log_file = open_run_log_for_append(&log_path)?;

    append_history_record(&mut log_file, &HistoryRecord::Operation(operation))
}

pub(crate) fn update_history_run_status(
    app: &AppHandle,
    run_id: &str,
    run_status: HistoryRunStatus,
) -> Result<(), String> {
    let history_root = history_root_for_app(app)?;
    let mut index = load_history_index(&history_root)?;
    let log_path = PathBuf::from(&index.find(run_id)?.history_file_path);

    let completed_at = Utc::now().to_rfc3339();
    let mut log_file = open_run_log_for_append(&log_path)?;
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

pub(crate) fn delete_history_run(app: &AppHandle, run_id: &str) -> Result<(), String> {
    let history_root = history_root_for_app(app)?;
    let mut index = load_history_index(&history_root)?;
    let log_path = PathBuf::from(&index.find(run_id)?.history_file_path);

    index.runs.retain(|entry| entry.run_id != run_id);
    write_history_index(&history_root, &index)?;

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
