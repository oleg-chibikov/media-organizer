use std::io;

use tempfile::tempdir;

use super::{entry_names, read, write};
use crate::safe_fs::{move_file::move_by_hard_link, move_file_no_replace, path_is_occupied};

#[test]
fn moves_a_file_and_drops_the_source_name() {
    let temp = tempdir().expect("temp dir");
    let (source, destination) = (temp.path().join("a.jpg"), temp.path().join("b.jpg"));
    write(&source, b"photo");

    move_file_no_replace(&source, &destination).expect("move");

    assert!(!source.exists());
    assert_eq!(read(&destination), b"photo");
}

#[test]
fn destination_that_appears_before_the_rename_is_never_replaced() {
    let temp = tempdir().expect("temp dir");
    let (source, destination) = (temp.path().join("a.jpg"), temp.path().join("b.jpg"));
    write(&source, b"photo");
    assert!(!path_is_occupied(&destination));
    write(&destination, b"arrived after the check");

    let error = move_file_no_replace(&source, &destination).expect_err("must refuse");

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(read(&source), b"photo");
    assert_eq!(read(&destination), b"arrived after the check");
}

#[test]
fn hard_link_fallback_moves_and_refuses_existing_destination() {
    let temp = tempdir().expect("temp dir");
    let source = temp.path().join("a.jpg");
    let (taken, free) = (temp.path().join("taken.jpg"), temp.path().join("free.jpg"));
    write(&source, b"photo");
    write(&taken, b"other");

    let error = move_by_hard_link(&source, &taken).expect_err("must refuse");
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(read(&taken), b"other");

    move_by_hard_link(&source, &free).expect("link move");
    assert!(!source.exists());
    assert_eq!(read(&free), b"photo");
}

#[test]
fn second_name_of_the_same_file_counts_as_taken() {
    let temp = tempdir().expect("temp dir");
    let (source, alias) = (temp.path().join("a.jpg"), temp.path().join("alias.jpg"));
    write(&source, b"photo");
    std::fs::hard_link(&source, &alias).expect("hard link");

    let error = move_file_no_replace(&source, &alias).expect_err("must refuse");

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(entry_names(temp.path()), ["a.jpg", "alias.jpg"]);
}

#[test]
fn case_and_unicode_aliases_never_lose_the_existing_file() {
    let temp = tempdir().expect("temp dir");
    for (existing_name, alias_name) in
        [("img.jpg", "IMG.JPG"), ("caf\u{e9}.jpg", "cafe\u{301}.jpg")]
    {
        let existing = temp.path().join(existing_name);
        let source = temp.path().join(format!("source-{existing_name}"));
        write(&existing, b"existing");
        write(&source, b"incoming");

        let moved = move_file_no_replace(&source, &temp.path().join(alias_name));

        assert_eq!(
            read(&existing),
            b"existing",
            "{alias_name} replaced {existing_name}"
        );
        match moved {
            Ok(()) => assert_eq!(read(&temp.path().join(alias_name)), b"incoming"),
            Err(error) => {
                assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
                assert_eq!(read(&source), b"incoming");
            }
        }
    }
}

#[cfg(unix)]
#[test]
fn dangling_symlink_at_destination_counts_as_taken() {
    let temp = tempdir().expect("temp dir");
    let (source, link) = (temp.path().join("a.jpg"), temp.path().join("link.jpg"));
    write(&source, b"photo");
    std::os::unix::fs::symlink(temp.path().join("missing.jpg"), &link).expect("symlink");

    assert!(path_is_occupied(&link));
    let error = move_file_no_replace(&source, &link).expect_err("must refuse");
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(read(&source), b"photo");
}
