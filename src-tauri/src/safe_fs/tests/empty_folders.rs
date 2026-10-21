use std::fs;

use tempfile::tempdir;

use super::{read, write};
use crate::safe_fs::remove_empty_folders;

#[test]
fn removes_nested_empty_folders_and_the_root_once_it_is_empty() {
    let temp = tempdir().expect("temp dir");
    let root = temp.path().join("todo");
    fs::create_dir_all(root.join("a").join("b").join("c")).expect("create deep");
    fs::create_dir_all(root.join("a").join("untouched")).expect("create sibling");
    fs::create_dir_all(root.join("d")).expect("create second branch");

    assert!(remove_empty_folders(&root, &temp.path().join("elsewhere")));

    assert!(!root.exists());
    assert!(temp.path().exists());
}

#[test]
fn keeps_every_file_and_the_folders_that_hold_them() {
    let temp = tempdir().expect("temp dir");
    let root = temp.path().join("todo");
    let busy = root.join("a").join("busy");
    fs::create_dir_all(&busy).expect("create busy");
    fs::create_dir_all(root.join("a").join("empty")).expect("create empty");
    fs::create_dir_all(root.join("b").join("empty")).expect("create second empty");
    write(&busy.join("keep.jpg"), b"photo");
    write(&root.join(".DS_Store"), b"finder");

    assert!(!remove_empty_folders(&root, &temp.path().join("elsewhere")));

    assert_eq!(read(&busy.join("keep.jpg")), b"photo");
    assert_eq!(read(&root.join(".DS_Store")), b"finder");
    assert!(!root.join("a").join("empty").exists());
    assert!(!root.join("b").exists());
}

#[test]
fn skips_the_kept_folder_and_everything_inside_it() {
    let temp = tempdir().expect("temp dir");
    let root = temp.path().join("todo");
    let processed = root.join("Processed");
    fs::create_dir_all(processed.join("2026")).expect("create processed");
    fs::create_dir_all(root.join("empty")).expect("create empty");

    assert!(!remove_empty_folders(&root, &processed));
    assert!(processed.join("2026").exists());
    assert!(!root.join("empty").exists());

    assert!(!remove_empty_folders(&root, &root));
    assert!(root.exists());
}

#[cfg(unix)]
#[test]
fn does_not_follow_links_out_of_the_root() {
    let temp = tempdir().expect("temp dir");
    let root = temp.path().join("todo");
    let outside = temp.path().join("outside");
    fs::create_dir_all(outside.join("inner")).expect("create outside");
    fs::create_dir_all(&root).expect("create root");
    std::os::unix::fs::symlink(&outside, root.join("link")).expect("symlink");

    assert!(!remove_empty_folders(&root, &temp.path().join("elsewhere")));

    assert!(outside.join("inner").exists());
    assert!(root.join("link").exists());
}
