use std::path::Path;

use chrono::Utc;

use super::{
    index::{load_history_index, with_index_lock, write_history_index},
    log_file::RunLogWriter,
    paths::{checked_run_log_path, history_runs_dir},
    run_lease::RunLease,
};
use crate::{
    models::{HistoryRecord, HistoryRunCompletedRecord, HistoryRunStatus},
    safe_fs,
};

/// Opens the log of an indexed run so undo/redo can append to it.
pub(super) fn open_run_log(history_root: &Path, run_id: &str) -> Result<RunLogWriter, String> {
    let index = load_history_index(history_root)?;
    let log_path = checked_run_log_path(history_root, &index.find(run_id)?.history_file_path)?;
    RunLogWriter::open_existing(&log_path)
}

pub(super) fn update_history_run_status(
    history_root: &Path,
    run_id: &str,
    run_status: HistoryRunStatus,
) -> Result<(), String> {
    with_index_lock(|| {
        let mut index = load_history_index(history_root)?;
        let index_entry = index
            .runs
            .iter_mut()
            .find(|entry| entry.run_id == run_id)
            .ok_or_else(|| format!("History run '{run_id}' was not found in index."))?;

        let completed_at = Utc::now().to_rfc3339();
        let log_path = checked_run_log_path(history_root, &index_entry.history_file_path)?;
        RunLogWriter::open_existing(&log_path)?.append(&HistoryRecord::RunCompleted(
            HistoryRunCompletedRecord {
                run_id: run_id.to_string(),
                completed_at: completed_at.clone(),
                run_status: run_status.clone(),
            },
        ))?;

        index_entry.completed_at = Some(completed_at);
        index_entry.run_status = run_status;
        write_history_index(history_root, &index)
    })
}

pub(super) fn delete_history_run(history_root: &Path, run_id: &str) -> Result<(), String> {
    let _lease = RunLease::acquire(run_id)?;
    let log_path = with_index_lock(|| {
        let mut index = load_history_index(history_root)?;
        let recorded = &index.find(run_id)?.history_file_path;
        let log_path = safe_fs::path_is_occupied(Path::new(recorded))
            .then(|| checked_run_log_path(history_root, recorded))
            .transpose()?;
        index.runs.retain(|entry| entry.run_id != run_id);
        write_history_index(history_root, &index).map(|()| log_path)
    })?;

    let Some(log_path) = log_path else {
        return Ok(());
    };
    safe_fs::remove_app_file(&log_path, &history_runs_dir(history_root)).map_err(|error| {
        format!(
            "Failed to remove history log file at '{}': {error}",
            log_path.display()
        )
    })
}
