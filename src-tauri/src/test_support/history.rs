use crate::models::{HistoryOperation, HistoryOperationKind, HistoryOperationStatus, MoveStrategy};

/// Succeeded rename operation; tests override paths, sizes and status as needed.
pub(crate) fn history_operation(
    run_id: &str,
    operation_id: &str,
    file_id: &str,
    operation_kind: HistoryOperationKind,
) -> HistoryOperation {
    HistoryOperation {
        operation_id: operation_id.to_string(),
        run_id: run_id.to_string(),
        file_id: file_id.to_string(),
        operation_kind,
        source_path: r"C:\todo\photo.jpg".to_string(),
        destination_path: r"C:\todo\Processed\photo.jpg".to_string(),
        expected_source_size: Some(10),
        expected_destination_size: Some(10),
        metadata_date: None,
        original_name: "photo".to_string(),
        move_strategy: MoveStrategy::Rename,
        status: HistoryOperationStatus::Succeeded,
        error: None,
        timestamp: "2026-05-24T01:03:03Z".to_string(),
    }
}
