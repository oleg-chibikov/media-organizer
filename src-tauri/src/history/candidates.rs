use std::collections::HashMap;

use crate::models::{HistoryOperation, HistoryOperationKind, HistoryOperationStatus, MoveStrategy};

/// Files whose latest successful operation left them in the processed destination.
pub(crate) fn collect_undoable_operations(
    operations: &[HistoryOperation],
) -> Vec<HistoryOperation> {
    let has_undo_redo_ops = operations.iter().any(|op| {
        matches!(
            op.operation_kind,
            HistoryOperationKind::Undo | HistoryOperationKind::Redo
        )
    });

    let undoable = if has_undo_redo_ops {
        latest_successful_by_file(operations, |kind| {
            matches!(
                kind,
                HistoryOperationKind::Process | HistoryOperationKind::Redo
            )
        })
    } else {
        succeeded_process_operations(operations)
    };
    sorted_by_operation_id(undoable)
}

/// Files whose latest successful operation moved them back to their original source.
pub(crate) fn collect_redoable_operations(
    operations: &[HistoryOperation],
) -> Vec<HistoryOperation> {
    let has_undo_ops = operations
        .iter()
        .any(|op| matches!(op.operation_kind, HistoryOperationKind::Undo));

    let redoable = if has_undo_ops {
        latest_successful_by_file(operations, |kind| {
            matches!(kind, HistoryOperationKind::Undo)
        })
    } else {
        succeeded_process_operations(operations)
    };
    sorted_by_operation_id(redoable)
}

fn latest_successful_by_file(
    operations: &[HistoryOperation],
    is_candidate_kind: impl Fn(&HistoryOperationKind) -> bool,
) -> Vec<HistoryOperation> {
    let mut last_succeeded_by_file = HashMap::<String, &HistoryOperation>::new();
    for operation in operations {
        if matches!(operation.status, HistoryOperationStatus::Succeeded) {
            last_succeeded_by_file.insert(operation.file_id.clone(), operation);
        }
    }

    last_succeeded_by_file
        .into_values()
        .filter(|operation| {
            is_candidate_kind(&operation.operation_kind) && has_replayable_strategy(operation)
        })
        .cloned()
        .collect()
}

fn succeeded_process_operations(operations: &[HistoryOperation]) -> Vec<HistoryOperation> {
    operations
        .iter()
        .filter(|op| {
            matches!(op.status, HistoryOperationStatus::Succeeded)
                && matches!(op.operation_kind, HistoryOperationKind::Process)
                && has_replayable_strategy(op)
        })
        .cloned()
        .collect()
}

fn has_replayable_strategy(operation: &HistoryOperation) -> bool {
    matches!(
        operation.move_strategy,
        MoveStrategy::Rename | MoveStrategy::CopyKeepSource
    )
}

fn sorted_by_operation_id(mut operations: Vec<HistoryOperation>) -> Vec<HistoryOperation> {
    operations.sort_by(|left, right| left.operation_id.cmp(&right.operation_id));
    operations
}
