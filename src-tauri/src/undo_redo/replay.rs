use tauri::AppHandle;

use super::outcome::{OperationOutcome, ReplaySummary};
use crate::{
    events, history,
    models::{HistoryIndexUpdatedEvent, HistoryOperation, HistoryRunStatus},
};

/// Executes each operation, appends its replay record to the run log and reports progress.
pub(super) fn replay_operations(
    app: &AppHandle,
    run_id: &str,
    operations: &[HistoryOperation],
    execute: impl Fn(&HistoryOperation) -> OperationOutcome,
    record: impl Fn(&str, &HistoryOperation, &OperationOutcome) -> HistoryOperation,
    emit_progress: impl Fn(u64, u64, HistoryOperation),
) -> ReplaySummary {
    let total_operations = operations.len() as u64;
    let mut summary = ReplaySummary::new(total_operations);

    for (index, operation) in operations.iter().enumerate() {
        let outcome = execute(operation);
        let history_operation = record(run_id, operation, &outcome);
        let _ = history::append_history_operation_for_run(app, run_id, history_operation.clone());
        summary.count(&history_operation.status);
        emit_progress(total_operations, index as u64 + 1, history_operation);
    }

    summary
}

/// Partial replays of either direction leave the run as `PartiallyUndone`.
pub(super) fn completion_status(
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

pub(super) fn finish_replay(
    app: &AppHandle,
    run_id: &str,
    initial_status: &HistoryRunStatus,
    final_status: HistoryRunStatus,
) {
    if final_status != *initial_status {
        let _ = history::update_history_run_status(app, run_id, final_status);
    }

    let _ = events::emit_history_index_updated(
        app,
        HistoryIndexUpdatedEvent {
            run_id: Some(run_id.to_string()),
        },
    );
}
