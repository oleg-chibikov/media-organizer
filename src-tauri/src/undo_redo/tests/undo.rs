use std::fs;

use tempfile::tempdir;

use super::{photo_paths, recorded_rename};
use crate::{
    models::{HistoryOperation, HistoryOperationKind, HistoryOperationStatus, MoveStrategy},
    undo_redo::{records::build_undo_history_operation, undo::execute_undo_operation},
};

#[test]
fn undoes_rename_operation_when_validation_passes() {
    let temp = tempdir().expect("temp dir");
    let (original_source, processed_destination) = photo_paths(temp.path());
    fs::create_dir_all(
        processed_destination
            .parent()
            .expect("destination should have parent"),
    )
    .expect("create destination parent");
    fs::write(&processed_destination, b"image-bytes").expect("write processed destination");

    let process_operation = recorded_rename(
        "run-1-op-1",
        HistoryOperationKind::Process,
        &original_source,
        &processed_destination,
        "2026-05-24T01:03:03Z",
    );

    let result = execute_undo_operation(&process_operation);
    let undo_operation = build_undo_history_operation("run-1", &process_operation, &result);

    assert!(matches!(result.status, HistoryOperationStatus::Succeeded));
    assert!(original_source.exists());
    assert!(!processed_destination.exists());
    assert!(matches!(
        undo_operation.operation_kind,
        HistoryOperationKind::Undo
    ));
    assert_eq!(
        undo_operation.source_path,
        process_operation.destination_path
    );
    assert_eq!(
        undo_operation.destination_path,
        process_operation.source_path
    );
}

#[test]
fn blocks_undo_when_rename_source_already_exists() {
    let temp = tempdir().expect("temp dir");
    let (original_source, processed_destination) = photo_paths(temp.path());
    fs::create_dir_all(
        processed_destination
            .parent()
            .expect("destination should have parent"),
    )
    .expect("create destination parent");
    fs::create_dir_all(original_source.parent().expect("source should have parent"))
        .expect("create source parent");
    fs::write(&processed_destination, b"image-bytes").expect("write processed destination");
    fs::write(&original_source, b"existing-source").expect("write existing source");

    let process_operation = recorded_rename(
        "run-1-op-1",
        HistoryOperationKind::Process,
        &original_source,
        &processed_destination,
        "2026-05-24T01:03:03Z",
    );

    let result = execute_undo_operation(&process_operation);
    let undo_operation = build_undo_history_operation("run-1", &process_operation, &result);

    assert!(matches!(result.status, HistoryOperationStatus::Blocked));
    assert!(undo_operation
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("already exists"));
    assert!(original_source.exists());
    assert!(processed_destination.exists());
}

#[test]
fn undo_of_a_copy_removes_it_only_when_it_matches_the_kept_source() {
    let temp = tempdir().expect("temp dir");
    let (original_source, copied_destination) = photo_paths(temp.path());
    for path in [&original_source, &copied_destination] {
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    }
    fs::write(&original_source, b"image-bytes").expect("write source");
    fs::write(&copied_destination, b"image-BYTES").expect("write edited copy");
    let copy_operation = HistoryOperation {
        move_strategy: MoveStrategy::CopyKeepSource,
        ..recorded_rename(
            "run-1-op-1",
            HistoryOperationKind::Process,
            &original_source,
            &copied_destination,
            "2026-05-24T01:03:03Z",
        )
    };

    let edited = execute_undo_operation(&copy_operation);
    assert!(matches!(edited.status, HistoryOperationStatus::Blocked));
    assert_eq!(
        fs::read(&copied_destination).expect("read copy"),
        b"image-BYTES"
    );

    fs::write(&copied_destination, b"image-bytes").expect("write matching copy");
    let matching = execute_undo_operation(&copy_operation);
    assert!(matches!(matching.status, HistoryOperationStatus::Succeeded));
    assert!(!copied_destination.exists());
    assert_eq!(
        fs::read(&original_source).expect("read source"),
        b"image-bytes"
    );
}
