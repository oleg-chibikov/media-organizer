use std::fs;

use tempfile::tempdir;

use crate::{mover::validate_move_safety_with_roots, test_support::media::photo_move_operation};

#[test]
fn safety_check_rejects_destination_outside_processed_folder() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("source.jpg");
    let destination = temp.path().join("outside.jpg");
    fs::write(&source, b"source").expect("write source");

    let checked = validate_move_safety_with_roots(
        temp.path(),
        &temp.path().join("Processed"),
        vec![photo_move_operation(&source, &destination)],
    );

    assert_eq!(checked.ready.len(), 0);
    assert_eq!(checked.skipped.len(), 1);
    assert!(checked.skipped[0]
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("outside the effective destination root"));
    assert!(source.exists());
}

#[test]
fn safety_check_allows_source_inside_processed_folder() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("Processed").join("source.jpg");
    let destination = temp
        .path()
        .join("Processed")
        .join("2026")
        .join("destination.jpg");
    fs::create_dir_all(source.parent().expect("source parent")).expect("create parent");
    fs::write(&source, b"source").expect("write source");

    let checked = validate_move_safety_with_roots(
        temp.path(),
        &temp.path().join("Processed"),
        vec![photo_move_operation(&source, &destination)],
    );

    assert_eq!(checked.ready.len(), 1);
    assert_eq!(checked.skipped.len(), 0);
    assert!(source.exists());
}

#[test]
fn safety_check_with_custom_root_rejects_destination_outside_effective_root() {
    let temp = tempdir().expect("temp dir");
    let source_root = temp.path().join("todo");
    let destination_root = temp.path().join("media-root");
    let source = source_root.join("source.jpg");
    let destination = temp.path().join("outside.jpg");
    fs::create_dir_all(&source_root).expect("create source root");
    fs::create_dir_all(&destination_root).expect("create destination root");
    fs::write(&source, b"source").expect("write source");

    let checked = validate_move_safety_with_roots(
        &source_root,
        &destination_root,
        vec![photo_move_operation(&source, &destination)],
    );

    assert_eq!(checked.ready.len(), 0);
    assert_eq!(checked.skipped.len(), 1);
    assert!(checked.skipped[0]
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("outside the effective destination root"));
    assert!(source.exists());
}

#[test]
fn safety_check_with_custom_root_allows_source_inside_effective_root() {
    let temp = tempdir().expect("temp dir");
    let source_root = temp.path().join("todo");
    let destination_root = temp.path().to_path_buf();
    let source = source_root.join("source.jpg");
    let destination = destination_root.join("2026").join("destination.jpg");
    fs::create_dir_all(&source_root).expect("create source root");
    fs::write(&source, b"source").expect("write source");

    let checked = validate_move_safety_with_roots(
        &source_root,
        &destination_root,
        vec![photo_move_operation(&source, &destination)],
    );

    assert_eq!(checked.ready.len(), 1);
    assert_eq!(checked.skipped.len(), 0);
    assert!(source.exists());
}
