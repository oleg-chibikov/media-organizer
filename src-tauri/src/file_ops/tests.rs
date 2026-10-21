use std::fs;

use tempfile::tempdir;

use super::{copy_into_new_file, files_have_same_bytes};

#[test]
fn compares_files_by_size_and_bytes() {
    let temp = tempdir().expect("temp dir");
    let original = temp.path().join("original.jpg");
    let same = temp.path().join("same.jpg");
    let edited = temp.path().join("edited.jpg");
    let longer = temp.path().join("longer.jpg");
    fs::write(&original, b"image-bytes").expect("write original");
    fs::write(&same, b"image-bytes").expect("write same");
    fs::write(&edited, b"image-BYTES").expect("write edited");
    fs::write(&longer, b"image-bytes!").expect("write longer");

    assert!(files_have_same_bytes(&original, &same).expect("compare same"));
    assert!(!files_have_same_bytes(&original, &edited).expect("compare edited"));
    assert!(!files_have_same_bytes(&original, &longer).expect("compare longer"));
}

#[test]
fn copy_into_new_file_refuses_to_overwrite() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("source.jpg");
    let destination = temp.path().join("destination.jpg");
    fs::write(&source, b"source").expect("write source");
    fs::write(&destination, b"existing").expect("write destination");

    let error = copy_into_new_file(&source, &destination).expect_err("copy should refuse");

    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(
        fs::read(&destination).expect("read destination"),
        b"existing"
    );
}

#[test]
fn copy_into_new_file_removes_partial_copy_when_source_is_missing() {
    let temp = tempdir().expect("temp dir");
    let destination = temp.path().join("destination.jpg");

    assert!(copy_into_new_file(&temp.path().join("missing.jpg"), &destination).is_err());
    assert!(!destination.exists());
}
