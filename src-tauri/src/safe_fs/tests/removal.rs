use std::fs;

use tempfile::tempdir;

use super::{read, write};
use crate::safe_fs::{remove_empty_dir_within, remove_verified_duplicate};

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

#[test]
fn removes_only_empty_folders_strictly_inside_the_root() {
    let temp = tempdir().expect("temp dir");
    let root = temp.path().join("todo");
    let (empty, busy) = (root.join("empty"), root.join("busy"));
    fs::create_dir_all(&empty).expect("create empty");
    fs::create_dir_all(&busy).expect("create busy");
    write(&busy.join("keep.jpg"), b"photo");
    let outside = temp.path().join("outside");
    fs::create_dir_all(&outside).expect("create outside");

    assert!(!remove_empty_dir_within(&root, &root));
    assert!(!remove_empty_dir_within(&busy, &root));
    assert!(!remove_empty_dir_within(&outside, &root));
    assert!(remove_empty_dir_within(&empty, &root));
    assert!(root.exists() && busy.exists() && outside.exists() && !empty.exists());
}

#[cfg(unix)]
#[test]
fn never_follows_links_out_of_the_root() {
    let temp = tempdir().expect("temp dir");
    let root = temp.path().join("todo");
    let outside = temp.path().join("outside");
    fs::create_dir_all(outside.join("inner")).expect("create outside");
    fs::create_dir_all(&root).expect("create root");
    std::os::unix::fs::symlink(&outside, root.join("link")).expect("symlink");

    assert!(!remove_empty_dir_within(
        &root.join("link").join("inner"),
        &root
    ));
    assert!(!remove_empty_dir_within(&root.join("link"), &root));
    assert!(outside.join("inner").exists());
    assert!(root.join("link").exists());
}
