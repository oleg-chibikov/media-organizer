use std::{fs, io};

use tempfile::tempdir;

use crate::{
    models::{MoveResultStatus, MoveStrategy},
    mover::{execute_move_with_rename, rename::execute_move_with_rename_internal},
    safe_fs::move_file_no_replace,
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
        |_src, _dest| Err(io::Error::from(io::ErrorKind::CrossesDevices)),
    );

    assert!(matches!(result.move_strategy, MoveStrategy::CopyKeepSource));
    assert!(matches!(result.result_status, MoveResultStatus::Copied));
    assert!(source.exists());
    assert!(destination.exists());
    assert_eq!(fs::read(&source).expect("read source"), b"source");
    assert_eq!(fs::read(&destination).expect("read destination"), b"source");
}

#[test]
fn destination_created_right_before_the_move_is_kept_and_source_stays() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("source.jpg");
    let destination = temp.path().join("destination.jpg");
    fs::write(&source, b"source").expect("write source");

    let result =
        execute_move_with_rename_internal(&photo_move_operation(&source, &destination), |s, d| {
            fs::write(d, b"intruder").expect("write intruder");
            move_file_no_replace(s, d)
        });

    assert!(matches!(result.result_status, MoveResultStatus::Skipped));
    assert_eq!(fs::read(&source).expect("read source"), b"source");
    assert_eq!(
        fs::read(&destination).expect("read destination"),
        b"intruder"
    );
}

#[test]
fn falls_back_to_verified_copy_when_no_replace_move_is_unsupported() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("source.jpg");
    let destination = temp.path().join("destination.jpg");
    fs::write(&source, b"source").expect("write source");

    let result = execute_move_with_rename_internal(
        &photo_move_operation(&source, &destination),
        |_src, _dest| Err(io::Error::from(io::ErrorKind::Unsupported)),
    );

    assert!(matches!(result.result_status, MoveResultStatus::Copied));
    assert_eq!(fs::read(&source).expect("read source"), b"source");
    assert_eq!(fs::read(&destination).expect("read destination"), b"source");
}

#[cfg(unix)]
#[test]
fn symlinked_source_is_not_moved() {
    let temp = tempdir().expect("temp dir");
    let target = temp.path().join("target.jpg");
    let link = temp.path().join("link.jpg");
    let destination = temp.path().join("destination.jpg");
    fs::write(&target, b"target").expect("write target");
    std::os::unix::fs::symlink(&target, &link).expect("symlink");

    let result = execute_move_with_rename(&photo_move_operation(&link, &destination));

    assert!(matches!(result.result_status, MoveResultStatus::Failed));
    assert!(link.exists() && !destination.exists());
}
