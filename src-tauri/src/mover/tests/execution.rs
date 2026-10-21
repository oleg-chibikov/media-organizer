use std::{fs, io};

use tempfile::tempdir;

use crate::{
    models::{MoveResultStatus, MoveStrategy},
    mover::{
        execute_move_with_rename, rename::execute_move_with_rename_internal,
        verification::verify_destination_matches_source,
    },
    test_support::media::photo_move_operation,
};

#[test]
fn rename_move_succeeds_and_removes_source() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("source.jpg");
    let destination = temp.path().join("Processed\\2026\\05\\23\\destination.jpg");
    fs::write(&source, b"source").expect("write source");

    let result = execute_move_with_rename(&photo_move_operation(&source, &destination));

    assert!(matches!(result.move_strategy, MoveStrategy::Rename));
    assert!(matches!(result.result_status, MoveResultStatus::Moved));
    assert!(!source.exists());
    assert!(destination.exists());
    assert_eq!(fs::read(&destination).expect("read destination"), b"source");
}

#[test]
fn rename_move_refuses_to_overwrite_existing_destination() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("source.jpg");
    let destination = temp.path().join("destination.jpg");
    fs::write(&source, b"source").expect("write source");
    fs::write(&destination, b"existing").expect("write destination");

    let result = execute_move_with_rename(&photo_move_operation(&source, &destination));

    assert!(matches!(result.move_strategy, MoveStrategy::NotAttempted));
    assert!(matches!(result.result_status, MoveResultStatus::Skipped));
    assert!(source.exists());
    assert_eq!(
        fs::read(&destination).expect("read destination"),
        b"existing"
    );
}

#[test]
fn falls_back_to_copy_keep_source_when_rename_reports_cross_device() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("source.jpg");
    let destination = temp.path().join("Processed\\2026\\05\\23\\destination.jpg");
    fs::write(&source, b"source").expect("write source");

    let result = execute_move_with_rename_internal(
        &photo_move_operation(&source, &destination),
        |_src, _dest| Err(io::Error::from_raw_os_error(17)),
    );

    assert!(matches!(result.move_strategy, MoveStrategy::CopyKeepSource));
    assert!(matches!(result.result_status, MoveResultStatus::Copied));
    assert!(source.exists());
    assert!(destination.exists());
    assert_eq!(fs::read(&source).expect("read source"), b"source");
    assert_eq!(fs::read(&destination).expect("read destination"), b"source");
}

#[test]
fn verification_fails_when_destination_size_differs() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("source.jpg");
    let destination = temp.path().join("destination.jpg");
    fs::write(&source, b"123456").expect("write source");
    fs::write(&destination, b"123").expect("write destination");

    let verify = verify_destination_matches_source(&source, &destination);

    assert!(verify.is_err());
    let error_message = verify.expect_err("expected verify failure");
    assert!(error_message.contains("size mismatch"));
}

#[test]
fn verification_fails_when_destination_content_differs() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("source.jpg");
    let destination = temp.path().join("destination.jpg");
    fs::write(&source, b"123456").expect("write source");
    fs::write(&destination, b"abcdef").expect("write destination");

    let verify = verify_destination_matches_source(&source, &destination);

    assert!(verify.is_err());
    let error_message = verify.expect_err("expected verify failure");
    assert!(error_message.contains("content mismatch"));
}
