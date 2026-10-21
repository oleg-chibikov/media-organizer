use super::{direction::ReplayDirection, outcome::ReplaySummary};
use crate::{
    events::{self, EventSink},
    history::{HistoryStore, RunLogWriter},
    models::{HistoryOperation, HistoryReplayProgressEvent, HistoryRunStatus},
};

/// An eligible undo or redo whose run log is already open for its replay records.
pub(crate) struct ReplayJob {
    direction: ReplayDirection,
    run_id: String,
    initial_status: HistoryRunStatus,
    operations: Vec<HistoryOperation>,
    log: RunLogWriter,
    history: HistoryStore,
}

/// Fails before any file is touched when the run is unknown, ineligible or its log cannot open.
pub(crate) fn prepare_replay(
    history: &HistoryStore,
    run_id: &str,
    direction: ReplayDirection,
) -> Result<ReplayJob, String> {
    let details = history.require_run(run_id)?;
    let operations = direction.eligible_operations(&details)?;
    let log = history.open_run_log(run_id)?;
    Ok(ReplayJob {
        direction,
        run_id: run_id.to_string(),
        initial_status: details.run.run_status,
        operations,
        log,
        history: history.clone(),
    })
}

impl ReplayJob {
    /// Replays each operation, logging and reporting it; blocks until the run is updated.
    pub(crate) fn run(mut self, sink: &impl EventSink) {
        let total_operations = self.operations.len() as u64;
        let mut summary = ReplaySummary::new(total_operations);

        for (index, operation) in self.operations.iter().enumerate() {
            let outcome = self.direction.execute(operation);
            let record = self.direction.record(&self.run_id, operation, &outcome);
            let logged = self.log.append_operation(record.clone());
            summary.count(&record.status);
            self.direction.emit_progress(
                sink,
                HistoryReplayProgressEvent {
                    run_id: self.run_id.clone(),
                    total_operations,
                    completed_operations: index as u64 + 1,
                    operation: record,
                },
            );
            if logged.is_err() {
                // Stop moving files that the history could no longer undo or redo.
                break;
            }
        }

        let final_status = completion_status(
            &self.initial_status,
            &summary,
            self.direction.fully_replayed_status(),
        );
        if final_status != self.initial_status {
            let _ = self.history.update_run_status(&self.run_id, final_status);
        }
        events::history_index_updated(sink, &self.run_id);
    }
}

/// Partial replays of either direction leave the run as `PartiallyUndone`.
fn completion_status(
    initial_status: &HistoryRunStatus,
    summary: &ReplaySummary,
    fully_replayed: HistoryRunStatus,
) -> HistoryRunStatus {
    if summary.succeeded == 0 {
        return initial_status.clone();
    }
    if summary.succeeded == summary.attempted {
        return fully_replayed;
    }
    HistoryRunStatus::PartiallyUndone
}
