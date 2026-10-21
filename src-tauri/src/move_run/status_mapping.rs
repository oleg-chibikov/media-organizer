use crate::models::{HistoryOperationStatus, HistoryRunStatus, MoveResultStatus};

pub(super) fn history_operation_status_for_move_result(
    result_status: &MoveResultStatus,
) -> HistoryOperationStatus {
    match result_status {
        MoveResultStatus::Moved | MoveResultStatus::Copied => HistoryOperationStatus::Succeeded,
        MoveResultStatus::Skipped => HistoryOperationStatus::Skipped,
        MoveResultStatus::Failed => HistoryOperationStatus::Failed,
    }
}

pub(super) fn history_run_status_for_completion(
    failed: u64,
    history_log_had_error: bool,
) -> HistoryRunStatus {
    if history_log_had_error {
        return HistoryRunStatus::Corrupt;
    }
    if failed > 0 {
        return HistoryRunStatus::CompletedWithErrors;
    }
    HistoryRunStatus::Completed
}
