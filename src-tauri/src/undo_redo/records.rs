use chrono::Utc;

use super::{outcome::OperationOutcome, redo::expected_redo_source_size};
use crate::{
    history,
    models::{HistoryOperation, HistoryOperationKind, MoveStrategy},
};

pub(super) fn build_undo_history_operation(
    run_id: &str,
    process_operation: &HistoryOperation,
    result: &OperationOutcome,
) -> HistoryOperation {
    let expected_destination_size = match process_operation.move_strategy {
        MoveStrategy::Rename => process_operation.expected_source_size,
        MoveStrategy::CopyKeepSource | MoveStrategy::NotAttempted => None,
    };

    HistoryOperation {
        operation_id: history::next_operation_id(run_id),
        run_id: run_id.to_string(),
        file_id: process_operation.file_id.clone(),
        operation_kind: HistoryOperationKind::Undo,
        source_path: process_operation.destination_path.clone(),
        destination_path: process_operation.source_path.clone(),
        expected_source_size: process_operation.expected_destination_size,
        expected_destination_size,
        metadata_date: process_operation.metadata_date.clone(),
        original_name: process_operation.original_name.clone(),
        move_strategy: process_operation.move_strategy.clone(),
        status: result.status.clone(),
        error: result.error.clone(),
        timestamp: Utc::now().to_rfc3339(),
    }
}

pub(super) fn build_redo_history_operation(
    run_id: &str,
    operation: &HistoryOperation,
    result: &OperationOutcome,
) -> HistoryOperation {
    let expected_source_size = expected_redo_source_size(operation);
    let (source_path, destination_path) = match operation.operation_kind {
        HistoryOperationKind::Process => (
            operation.source_path.clone(),
            operation.destination_path.clone(),
        ),
        _ => (
            operation.destination_path.clone(),
            operation.source_path.clone(),
        ),
    };

    HistoryOperation {
        operation_id: history::next_operation_id(run_id),
        run_id: run_id.to_string(),
        file_id: operation.file_id.clone(),
        operation_kind: HistoryOperationKind::Redo,
        source_path,
        destination_path,
        expected_source_size,
        expected_destination_size: expected_source_size,
        metadata_date: operation.metadata_date.clone(),
        original_name: operation.original_name.clone(),
        move_strategy: operation.move_strategy.clone(),
        status: result.status.clone(),
        error: result.error.clone(),
        timestamp: Utc::now().to_rfc3339(),
    }
}
