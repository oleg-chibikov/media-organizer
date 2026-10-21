use crate::models::{HistoryOperation, HistoryRecord, HistoryRun, HistoryRunCompletedRecord};

/// Records accepted so far from one run log, plus any damage found while reading it.
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

impl ParsedRunLog {
    /// Accepts a record of the expected run, or returns the mismatch label and foreign run id.
    pub(super) fn apply(
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

    pub(super) fn mark_corrupt(&mut self, message: String) {
        self.corrupt = true;
        self.recovery_error = Some(message);
    }
}
