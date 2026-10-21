use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use crate::models::HistoryRecord;

pub(super) fn create_run_log(log_path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(log_path)
        .map_err(|error| open_error(log_path, error))
}

pub(super) fn open_run_log_for_append(log_path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .append(true)
        .open(log_path)
        .map_err(|error| open_error(log_path, error))
}

/// Writes one JSONL record and flushes so a crash loses at most a partial trailing line.
pub(super) fn append_history_record(file: &mut File, record: &HistoryRecord) -> Result<(), String> {
    let line = serde_json::to_string(record)
        .map_err(|error| format!("Failed to serialize history record: {error}"))?;
    file.write_all(line.as_bytes())
        .map_err(|error| format!("Failed to append history record: {error}"))?;
    file.write_all(b"\n")
        .map_err(|error| format!("Failed to append history record newline: {error}"))?;
    file.flush()
        .map_err(|error| format!("Failed to flush history record: {error}"))
}

fn open_error(log_path: &Path, error: std::io::Error) -> String {
    format!(
        "Failed to open history run log at '{}': {error}",
        log_path.display()
    )
}
