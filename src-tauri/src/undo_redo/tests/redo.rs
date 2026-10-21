use std::fs;

use tempfile::tempdir;

use super::{photo_paths, recorded_rename};
use crate::{
    models::{HistoryOperationKind, HistoryOperationStatus},
    undo_redo::{records::build_redo_history_operation, redo::execute_redo_operation},
};

#[test]
fn redoes_rename_operation_when_validation_passes() {
    let temp = tempdir().expect("temp dir");
    let (original_source, processed_destination) = photo_paths(temp.path());
    fs::create_dir_all(original_source.parent().expect("source should have parent"))
        .expect("create source parent");
    fs::write(&original_source, b"image-bytes").expect("write original source");

    let undo_operation = recorded_rename(
        "run-1-op-2",
        HistoryOperationKind::Undo,
        &processed_destination,
        &original_source,
        "2026-05-24T01:08:03Z",
    );

    let result = execute_redo_operation(&undo_operation);
    let redo_operation = build_redo_history_operation("run-1", &undo_operation, &result);

    assert!(matches!(result.status, HistoryOperationStatus::Succeeded));
    assert!(!original_source.exists());
    assert!(processed_destination.exists());
    assert!(matches!(
        redo_operation.operation_kind,
        HistoryOperationKind::Redo
    ));
    assert_eq!(redo_operation.source_path, undo_operation.destination_path);
    assert_eq!(redo_operation.destination_path, undo_operation.source_path);
}

#[test]
fn redoes_rename_operation_from_process_record_for_legacy_history() {
    let temp = tempdir().expect("temp dir");
    let (original_source, processed_destination) = photo_paths(temp.path());
    fs::create_dir_all(original_source.parent().expect("source should have parent"))
        .expect("create source parent");
    fs::write(&original_source, b"image-bytes").expect("write original source");

    let process_operation = recorded_rename(
        "run-1-op-1",
        HistoryOperationKind::Process,
        &original_source,
        &processed_destination,
        "2026-05-24T01:03:03Z",
    );

    let result = execute_redo_operation(&process_operation);
    let redo_operation = build_redo_history_operation("run-1", &process_operation, &result);

    assert!(matches!(result.status, HistoryOperationStatus::Succeeded));
    assert!(!original_source.exists());
    assert!(processed_destination.exists());
    assert_eq!(redo_operation.source_path, process_operation.source_path);
    assert_eq!(
        redo_operation.destination_path,
        process_operation.destination_path
    );
}

#[test]
fn blocks_redo_when_recorded_destination_already_exists() {
    let temp = tempdir().expect("temp dir");
    let (original_source, processed_destination) = photo_paths(temp.path());
    fs::create_dir_all(original_source.parent().expect("source should have parent"))
        .expect("create source parent");
    fs::create_dir_all(
        processed_destination
            .parent()
            .expect("destination should have parent"),
    )
    .expect("create destination parent");
    fs::write(&original_source, b"image-bytes").expect("write original source");
    fs::write(&processed_destination, b"existing-destination").expect("write existing destination");

    let undo_operation = recorded_rename(
        "run-1-op-2",
        HistoryOperationKind::Undo,
        &processed_destination,
        &original_source,
        "2026-05-24T01:08:03Z",
    );

    let result = execute_redo_operation(&undo_operation);
    let redo_operation = build_redo_history_operation("run-1", &undo_operation, &result);

    assert!(matches!(result.status, HistoryOperationStatus::Blocked));
    assert!(redo_operation
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("already exists"));
    assert!(original_source.exists());
    assert!(processed_destination.exists());
}
