use std::fs;

use tempfile::tempdir;

use super::{read, write};
use crate::safe_fs::remove_verified_duplicate;

#[test]
fn removes_a_duplicate_only_when_the_original_holds_the_same_bytes() {
    let temp = tempdir().expect("temp dir");
    let original = temp.path().join("original.jpg");
    let (copy, edited) = (temp.path().join("copy.jpg"), temp.path().join("edited.jpg"));
    write(&original, b"photo");
    write(&copy, b"photo");
    write(&edited, b"PHOTO");

    assert!(remove_verified_duplicate(&edited, &original).is_err());
    assert_eq!(read(&edited), b"PHOTO");

    remove_verified_duplicate(&copy, &original).expect("remove duplicate");
    assert!(!copy.exists());
    assert_eq!(read(&original), b"photo");
}

#[test]
fn refuses_to_remove_a_second_name_of_the_same_file() {
    let temp = tempdir().expect("temp dir");
    let (original, alias) = (temp.path().join("a.jpg"), temp.path().join("alias.jpg"));
    write(&original, b"photo");
    fs::hard_link(&original, &alias).expect("hard link");

    assert!(remove_verified_duplicate(&alias, &original).is_err());
    assert!(alias.exists());
}

#[cfg(unix)]
#[test]
fn refuses_to_remove_a_symlink_pointing_at_the_original() {
    let temp = tempdir().expect("temp dir");
    let (original, link) = (temp.path().join("a.jpg"), temp.path().join("link.jpg"));
    write(&original, b"photo");
    std::os::unix::fs::symlink(&original, &link).expect("symlink");

    assert!(remove_verified_duplicate(&original, &link).is_err());
    assert!(remove_verified_duplicate(&link, &original).is_err());
    assert_eq!(read(&original), b"photo");
}
