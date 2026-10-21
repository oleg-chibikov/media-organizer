use std::io;

use tempfile::tempdir;

use super::{entry_names, read, write};
use crate::safe_fs::{
    content_check::verify_same_content, copy_verified_no_replace, move_file_no_replace,
    staged_copy::copy_verified_with_publisher,
};

#[test]
fn copies_verified_bytes_keeps_source_and_leaves_no_temp_file() {
    let temp = tempdir().expect("temp dir");
    let (source, destination) = (temp.path().join("a.jpg"), temp.path().join("b.jpg"));
    write(&source, b"photo");

    copy_verified_no_replace(&source, &destination).expect("copy");

    assert_eq!(read(&source), b"photo");
    assert_eq!(read(&destination), b"photo");
    assert_eq!(entry_names(temp.path()), ["a.jpg", "b.jpg"]);
}

#[test]
fn refuses_an_existing_destination() {
    let temp = tempdir().expect("temp dir");
    let (source, destination) = (temp.path().join("a.jpg"), temp.path().join("b.jpg"));
    write(&source, b"photo");
    write(&destination, b"existing");

    let error = copy_verified_no_replace(&source, &destination).expect_err("must refuse");

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(read(&destination), b"existing");
    assert_eq!(entry_names(temp.path()), ["a.jpg", "b.jpg"]);
}

#[test]
fn destination_that_appears_during_the_copy_is_never_replaced() {
    let temp = tempdir().expect("temp dir");
    let (source, destination) = (temp.path().join("a.jpg"), temp.path().join("b.jpg"));
    write(&source, b"photo");

    let error = copy_verified_with_publisher(&source, &destination, |staged, target| {
        write(target, b"arrived during copy");
        move_file_no_replace(staged, target)
    })
    .expect_err("must refuse");

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(read(&source), b"photo");
    assert_eq!(read(&destination), b"arrived during copy");
    assert_eq!(entry_names(temp.path()), ["a.jpg", "b.jpg"]);
}

#[test]
fn publishes_by_exclusive_copy_when_rename_and_links_are_unsupported() {
    let temp = tempdir().expect("temp dir");
    let (source, destination) = (temp.path().join("a.jpg"), temp.path().join("b.jpg"));
    write(&source, b"photo");

    copy_verified_with_publisher(&source, &destination, |_, _| {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    })
    .expect("fallback copy");

    assert_eq!(read(&destination), b"photo");
    assert_eq!(entry_names(temp.path()), ["a.jpg", "b.jpg"]);
}

#[test]
fn failed_copy_removes_only_its_own_temp_file() {
    let temp = tempdir().expect("temp dir");
    let destination = temp.path().join("b.jpg");

    assert!(copy_verified_no_replace(&temp.path().join("missing.jpg"), &destination).is_err());

    assert!(entry_names(temp.path()).is_empty());
}

#[test]
fn verification_reports_size_and_content_mismatches() {
    let temp = tempdir().expect("temp dir");
    let original = temp.path().join("a.jpg");
    let (shorter, edited) = (temp.path().join("short.jpg"), temp.path().join("edit.jpg"));
    write(&original, b"123456");
    write(&shorter, b"123");
    write(&edited, b"abcdef");

    let size = verify_same_content(&original, &shorter).expect_err("size differs");
    let content = verify_same_content(&original, &edited).expect_err("content differs");

    assert!(size.to_string().contains("size mismatch"));
    assert!(content.to_string().contains("content mismatch"));
    assert!(verify_same_content(&original, &original).is_ok());
}
