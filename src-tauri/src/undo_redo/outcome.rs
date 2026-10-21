use std::{fs, path::Path};

use crate::models::HistoryOperationStatus;

#[derive(Debug, Clone)]
pub(super) struct OperationOutcome {
    pub status: HistoryOperationStatus,
    pub error: Option<String>,
}

impl OperationOutcome {
    pub(super) fn succeeded() -> Self {
        Self {
            status: HistoryOperationStatus::Succeeded,
            error: None,
        }
    }

    pub(super) fn blocked(error: impl Into<String>) -> Self {
        Self {
            status: HistoryOperationStatus::Blocked,
            error: Some(error.into()),
        }
    }

    pub(super) fn failed(error: impl Into<String>) -> Self {
        Self {
            status: HistoryOperationStatus::Failed,
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ReplaySummary {
    pub attempted: u64,
    pub succeeded: u64,
    pub blocked: u64,
    pub failed: u64,
}

impl ReplaySummary {
    pub(super) fn new(attempted: u64) -> Self {
        Self {
            attempted,
            succeeded: 0,
            blocked: 0,
            failed: 0,
        }
    }

    pub(super) fn count(&mut self, status: &HistoryOperationStatus) {
        match status {
            HistoryOperationStatus::Succeeded => self.succeeded += 1,
            HistoryOperationStatus::Blocked => self.blocked += 1,
            HistoryOperationStatus::Failed => self.failed += 1,
            HistoryOperationStatus::Skipped => {}
        }
    }
}

/// Blocks the replay when the file at `path` no longer has the recorded size.
pub(super) fn ensure_expected_size(
    path: &Path,
    expected_size: Option<u64>,
    action: &str,
    subject: &str,
) -> Result<(), OperationOutcome> {
    let Some(expected_size) = expected_size else {
        return Ok(());
    };

    match fs::metadata(path) {
        Ok(meta) if meta.len() == expected_size => Ok(()),
        Ok(meta) => Err(OperationOutcome::blocked(format!(
            "{action} blocked: {subject} size mismatch (expected {expected_size}, found {}).",
            meta.len()
        ))),
        Err(error) => Err(OperationOutcome::blocked(format!(
            "{action} blocked: cannot read {subject} metadata: {error}"
        ))),
    }
}
