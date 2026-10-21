use std::{collections::HashMap, fs};

use tempfile::tempdir;

use crate::{
    models::{
        ConflictStatus, FileKind, FileRecord, HistoryOperationKind, HistoryOperationStatus,
        HistoryRunStatus, MetadataResult, MetadataStatus, MovePlan, MoveResult, MoveResultStatus,
        MoveStrategy, PlanStatus, ScanStatus,
    },
    move_run::{
        history_entry::{
            build_history_operation_for_result, history_operation_status_for_move_result,
            history_run_status_for_completion,
        },
        lookups::RunLookups,
    },
};

#[test]
fn builds_processing_history_operation_with_required_fields() {
    let temp = tempdir().expect("temp dir");
    let destination_path = temp.path().join("destination.jpg");
    fs::write(&destination_path, b"data").expect("write destination");

    let result = MoveResult {
        file_id: "photo-1".to_string(),
        attempted_source_path: r"C:\todo\photo-1.jpg".to_string(),
        attempted_destination_path: destination_path.to_string_lossy().to_string(),
        move_strategy: MoveStrategy::Rename,
        result_status: MoveResultStatus::Moved,
        error: None,
    };
    let metadata = MetadataResult {
        file_id: "photo-1".to_string(),
        chosen_date: Some("2026-05-24T01:02:03Z".to_string()),
        chosen_date_source: Some("DateTimeOriginal".to_string()),
        date_kind: None,
        raw_metadata_date: None,
        device_name: None,
        metadata_status: MetadataStatus::Ready,
        error: None,
    };
    let plan = MovePlan {
        file_id: "photo-1".to_string(),
        original_name: "IMG_0001".to_string(),
        destination_dir: r"C:\todo\Processed\2026\05\24".to_string(),
        destination_filename: "IMG_0001.jpg".to_string(),
        destination_path: destination_path.to_string_lossy().to_string(),
        conflict_status: ConflictStatus::None,
        plan_status: PlanStatus::Ready,
    };
    let file = FileRecord {
        id: "photo-1".to_string(),
        source_path: r"C:\todo\photo-1.jpg".to_string(),
        relative_path: "photo-1.jpg".to_string(),
        extension: "jpg".to_string(),
        original_name: "fallback_name".to_string(),
        kind: FileKind::Photo,
        scan_status: ScanStatus::Discovered,
    };

    let files = [file];
    let metadata_results = [metadata];
    let plans = [plan];
    let lookups = RunLookups::new(&files, &metadata_results, &plans);
    let expected_source_sizes = HashMap::from([("photo-1", Some(4096_u64))]);

    let operation =
        build_history_operation_for_result("run-1", &result, &lookups, &expected_source_sizes);

    assert_eq!(operation.run_id, "run-1");
    assert!(matches!(
        operation.operation_kind,
        HistoryOperationKind::Process
    ));
    assert_eq!(operation.file_id, "photo-1");
    assert_eq!(operation.source_path, r"C:\todo\photo-1.jpg");
    assert_eq!(
        operation.destination_path,
        destination_path.to_string_lossy()
    );
    assert_eq!(operation.expected_source_size, Some(4096));
    assert_eq!(operation.expected_destination_size, Some(4));
    assert_eq!(
        operation.metadata_date.as_deref(),
        Some("2026-05-24T01:02:03Z")
    );
    assert_eq!(operation.original_name, "IMG_0001");
    assert!(matches!(operation.move_strategy, MoveStrategy::Rename));
    assert!(matches!(
        operation.status,
        HistoryOperationStatus::Succeeded
    ));
    assert!(operation.error.is_none());
    assert!(!operation.timestamp.is_empty());
}

#[test]
fn maps_move_result_statuses_to_history_operation_statuses() {
    assert!(matches!(
        history_operation_status_for_move_result(&MoveResultStatus::Moved),
        HistoryOperationStatus::Succeeded
    ));
    assert!(matches!(
        history_operation_status_for_move_result(&MoveResultStatus::Copied),
        HistoryOperationStatus::Succeeded
    ));
    assert!(matches!(
        history_operation_status_for_move_result(&MoveResultStatus::Skipped),
        HistoryOperationStatus::Skipped
    ));
    assert!(matches!(
        history_operation_status_for_move_result(&MoveResultStatus::Failed),
        HistoryOperationStatus::Failed
    ));
}

#[test]
fn chooses_history_run_status_from_error_state() {
    assert!(matches!(
        history_run_status_for_completion(0, false),
        HistoryRunStatus::Completed
    ));
    assert!(matches!(
        history_run_status_for_completion(1, false),
        HistoryRunStatus::CompletedWithErrors
    ));
    assert!(matches!(
        history_run_status_for_completion(0, true),
        HistoryRunStatus::Corrupt
    ));
}
