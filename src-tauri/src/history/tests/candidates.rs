use crate::{
    history::{collect_redoable_operations, collect_undoable_operations},
    models::{HistoryOperation, HistoryOperationKind, HistoryOperationStatus, MoveStrategy},
    test_support::history::history_operation,
};

fn process(run_id: &str, operation_id: &str, file_id: &str) -> HistoryOperation {
    history_operation(run_id, operation_id, file_id, HistoryOperationKind::Process)
}

/// A succeeded replay moving the file between the processed and original paths.
fn replay(
    run_id: &str,
    operation_id: &str,
    operation_kind: HistoryOperationKind,
    timestamp: &str,
) -> HistoryOperation {
    let (source_path, destination_path) = match operation_kind {
        HistoryOperationKind::Undo => (r"C:\todo\Processed\photo.jpg", r"C:\todo\photo.jpg"),
        _ => (r"C:\todo\photo.jpg", r"C:\todo\Processed\photo.jpg"),
    };
    HistoryOperation {
        source_path: source_path.to_string(),
        destination_path: destination_path.to_string(),
        timestamp: timestamp.to_string(),
        ..history_operation(run_id, operation_id, "file-1", operation_kind)
    }
}

#[test]
fn collects_undoable_operations_for_supported_strategies_only() {
    let operations = vec![
        HistoryOperation {
            source_path: r"C:\todo\rename.jpg".to_string(),
            destination_path: r"C:\todo\Processed\rename.jpg".to_string(),
            original_name: "rename".to_string(),
            ..process("run-1", "run-1-01", "rename-file")
        },
        HistoryOperation {
            source_path: r"C:\todo\copy.jpg".to_string(),
            destination_path: r"C:\todo\Processed\copy.jpg".to_string(),
            expected_source_size: Some(11),
            expected_destination_size: Some(11),
            original_name: "copy".to_string(),
            move_strategy: MoveStrategy::CopyKeepSource,
            timestamp: "2026-05-24T01:04:03Z".to_string(),
            ..process("run-1", "run-1-02", "copy-file")
        },
        HistoryOperation {
            source_path: r"C:\todo\skip.jpg".to_string(),
            destination_path: r"C:\todo\Processed\skip.jpg".to_string(),
            expected_source_size: Some(12),
            expected_destination_size: Some(12),
            original_name: "skip".to_string(),
            status: HistoryOperationStatus::Skipped,
            timestamp: "2026-05-24T01:05:03Z".to_string(),
            ..process("run-1", "run-1-03", "skipped-file")
        },
    ];

    let undoable = collect_undoable_operations(&operations);
    assert_eq!(undoable.len(), 2);
    assert!(undoable
        .iter()
        .any(|operation| operation.file_id == "rename-file"));
    assert!(undoable
        .iter()
        .any(|operation| operation.file_id == "copy-file"));
}

#[test]
fn excludes_successfully_undone_operations_from_undo_candidates() {
    let operations = vec![
        process("run-2", "run-2-01", "file-1"),
        replay(
            "run-2",
            "run-2-02",
            HistoryOperationKind::Undo,
            "2026-05-24T01:06:03Z",
        ),
    ];

    let undoable = collect_undoable_operations(&operations);
    assert!(undoable.is_empty());
}

#[test]
fn collects_redoable_operations_when_latest_success_is_undo() {
    let operations = vec![
        process("run-3", "run-3-01", "file-1"),
        replay(
            "run-3",
            "run-3-02",
            HistoryOperationKind::Undo,
            "2026-05-24T01:06:03Z",
        ),
    ];

    let redoable = collect_redoable_operations(&operations);
    assert_eq!(redoable.len(), 1);
    assert_eq!(redoable[0].file_id, "file-1");
    assert!(matches!(
        redoable[0].operation_kind,
        HistoryOperationKind::Undo
    ));
}

#[test]
fn excludes_successfully_redone_operations_from_redo_candidates() {
    let operations = vec![
        replay(
            "run-4",
            "run-4-01",
            HistoryOperationKind::Undo,
            "2026-05-24T01:06:03Z",
        ),
        replay(
            "run-4",
            "run-4-02",
            HistoryOperationKind::Redo,
            "2026-05-24T01:07:03Z",
        ),
    ];

    let redoable = collect_redoable_operations(&operations);
    assert!(redoable.is_empty());
}
