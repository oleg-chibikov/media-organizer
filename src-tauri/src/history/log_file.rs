use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use crate::models::{HistoryOperation, HistoryRecord};

/// An open run log; each append writes one flushed JSONL record.
#[derive(Debug)]
pub(crate) struct RunLogWriter {
    file: File,
}

impl RunLogWriter {
    pub(super) fn create(log_path: &Path) -> Result<Self, String> {
        OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(log_path)
            .map(|file| Self { file })
            .map_err(|error| open_error(log_path, error))
    }

    pub(super) fn open_existing(log_path: &Path) -> Result<Self, String> {
        OpenOptions::new()
            .append(true)
            .open(log_path)
            .map(|file| Self { file })
            .map_err(|error| open_error(log_path, error))
    }

    /// A writer whose appends all fail, for tests of a log that stops recording.
    #[cfg(test)]
    pub(super) fn read_only_for_tests(log_path: &Path) -> Self {
        let file = File::open(log_path).expect("open run log read-only");
        Self { file }
    }

    pub(crate) fn append_operation(&mut self, operation: HistoryOperation) -> Result<(), String> {
        self.append(&HistoryRecord::Operation(operation))
    }

    pub(super) fn append(&mut self, record: &HistoryRecord) -> Result<(), String> {
        append_history_record(&mut self.file, record)
    }
}

/// Writes one JSONL record and flushes so a crash loses at most a partial trailing line.
/// The line and its newline go out in one write, so two appenders can't split a record.
pub(super) fn append_history_record(file: &mut File, record: &HistoryRecord) -> Result<(), String> {
    let mut line = serde_json::to_vec(record)
        .map_err(|error| format!("Failed to serialize history record: {error}"))?;
    line.push(b'\n');
    file.write_all(&line)
        .map_err(|error| format!("Failed to append history record: {error}"))?;
    file.flush()
        .map_err(|error| format!("Failed to flush history record: {error}"))
}

fn open_error(log_path: &Path, error: std::io::Error) -> String {
    format!(
        "Failed to open history run log at '{}': {error}",
        log_path.display()
    )
}
