use std::{fs::File, path::Path};

use super::log_lines::{LineDecodeError, RunLogLines};
use crate::models::{HistoryOperation, HistoryRecord, HistoryRun, HistoryRunCompletedRecord};

#[derive(Default)]
pub(super) struct ParsedRunLog {
    pub started_run: Option<HistoryRun>,
    pub operations: Vec<HistoryOperation>,
    pub completed_record: Option<HistoryRunCompletedRecord>,
    pub partial: bool,
    pub incomplete: bool,
    pub corrupt: bool,
    pub recovery_error: Option<String>,
}

/// Replays a run log, stopping at the first truncated, malformed or foreign record.
pub(super) fn parse_run_log_records(
    history_file_path: &Path,
    expected_run_id: &str,
) -> ParsedRunLog {
    let mut parsed = ParsedRunLog::default();

    let file = match File::open(history_file_path) {
        Ok(file) => file,
        Err(error) => {
            parsed.mark_corrupt(format!(
                "Failed to open history run log at '{}': {error}",
                history_file_path.display()
            ));
            return parsed;
        }
    };

    let mut lines = RunLogLines::new(file);
    loop {
        let line = match lines.next_line() {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(error) => {
                parsed.mark_corrupt(format!(
                    "Failed reading history run log at '{}': {error}",
                    history_file_path.display()
                ));
                return parsed;
            }
        };

        let record = match line.decode(history_file_path) {
            Ok(record) => record,
            Err(LineDecodeError::Truncated) => {
                parsed.partial = true;
                break;
            }
            Err(LineDecodeError::Malformed(message)) => {
                parsed.mark_corrupt(message);
                break;
            }
        };

        if let Err((label, found_run_id)) = parsed.apply(record, expected_run_id) {
            parsed.mark_corrupt(format!(
                "{label} at line {} in '{}': expected '{expected_run_id}', found '{found_run_id}'",
                line.number,
                history_file_path.display()
            ));
            break;
        }
    }

    parsed.incomplete = parsed.started_run.is_none() || parsed.completed_record.is_none();
    parsed
}

impl ParsedRunLog {
    /// Accepts a record of the expected run, or returns the mismatch label and foreign run id.
    fn apply(
        &mut self,
        record: HistoryRecord,
        expected_run_id: &str,
    ) -> Result<(), (&'static str, String)> {
        match record {
            HistoryRecord::RunStarted { run } => {
                if run.run_id != expected_run_id {
                    return Err(("Run id mismatch", run.run_id));
                }
                self.started_run = Some(run);
            }
            HistoryRecord::Operation(operation) => {
                if operation.run_id != expected_run_id {
                    return Err(("Operation run id mismatch", operation.run_id));
                }
                self.operations.push(operation);
            }
            HistoryRecord::RunCompleted(record) => {
                if record.run_id != expected_run_id {
                    return Err(("Completion run id mismatch", record.run_id));
                }
                self.completed_record = Some(record);
            }
        }
        Ok(())
    }

    fn mark_corrupt(&mut self, message: String) {
        self.corrupt = true;
        self.recovery_error = Some(message);
    }
}
