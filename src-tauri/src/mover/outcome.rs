use super::MoveOperation;
use crate::models::{MoveResult, MoveResultStatus, MoveStrategy};

pub(super) fn skipped_result(
    file_id: &str,
    source_path: &str,
    destination_path: &str,
    reason: &str,
) -> MoveResult {
    MoveResult {
        file_id: file_id.to_string(),
        attempted_source_path: source_path.to_string(),
        attempted_destination_path: destination_path.to_string(),
        move_strategy: MoveStrategy::NotAttempted,
        result_status: MoveResultStatus::Skipped,
        error: Some(reason.to_string()),
    }
}

pub(super) fn skipped_operation(operation: &MoveOperation, reason: &str) -> MoveResult {
    skipped_result(
        &operation.file_id,
        &operation.source_path,
        &operation.destination_path,
        reason,
    )
}

pub(super) fn attempted_result(
    operation: &MoveOperation,
    move_strategy: MoveStrategy,
    result_status: MoveResultStatus,
    error: Option<String>,
) -> MoveResult {
    MoveResult {
        file_id: operation.file_id.clone(),
        attempted_source_path: operation.source_path.clone(),
        attempted_destination_path: operation.destination_path.clone(),
        move_strategy,
        result_status,
        error,
    }
}
