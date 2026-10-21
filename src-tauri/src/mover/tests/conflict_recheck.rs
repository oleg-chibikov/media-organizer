use std::fs;

use tempfile::tempdir;

use crate::{
    models::MoveResultStatus, mover::recheck_destination_conflicts,
    test_support::media::photo_move_operation,
};

#[test]
fn recheck_skips_operation_when_destination_exists() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("source.jpg");
    let destination = temp.path().join("destination.jpg");
    fs::write(&source, b"source").expect("write source");
    fs::write(&destination, b"existing").expect("write destination");

    let rechecked =
        recheck_destination_conflicts(vec![photo_move_operation(&source, &destination)]);

    assert_eq!(rechecked.ready.len(), 0);
    assert_eq!(rechecked.skipped.len(), 1);
    assert_eq!(rechecked.skipped[0].file_id, "photo_1");
    assert!(matches!(
        rechecked.skipped[0].result_status,
        MoveResultStatus::Skipped
    ));
}

#[test]
fn recheck_keeps_operation_when_destination_is_available() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("source.jpg");
    let destination = temp.path().join("destination.jpg");
    fs::write(&source, b"source").expect("write source");

    let rechecked =
        recheck_destination_conflicts(vec![photo_move_operation(&source, &destination)]);

    assert_eq!(rechecked.ready.len(), 1);
    assert_eq!(rechecked.skipped.len(), 0);
    assert_eq!(rechecked.ready[0].file_id, "photo_1");
}
