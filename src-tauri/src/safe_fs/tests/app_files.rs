use std::fs;

use tempfile::tempdir;

use super::{entry_names, read, write};
use crate::safe_fs::{ensure_inside_dir, remove_app_file, replace_app_file};

#[test]
fn replaces_contents_and_leaves_no_temp_file() {
    let temp = tempdir().expect("temp dir");
    let path = temp.path().join("index.json");
    write(&path, b"old");

    replace_app_file(&path, b"new").expect("replace");

    assert_eq!(read(&path), b"new");
    assert_eq!(entry_names(temp.path()), ["index.json"]);
}

#[cfg(unix)]
#[test]
fn never_writes_through_a_planted_temp_symlink() {
    let temp = tempdir().expect("temp dir");
    let media = temp.path().join("photo.jpg");
    write(&media, b"photo");
    let path = temp.path().join("index.json");
    let planted = temp
        .path()
        .join(format!(".index.json.{}-0.tmp", std::process::id()));
    std::os::unix::fs::symlink(&media, &planted).expect("symlink");

    replace_app_file(&path, b"new").expect("replace");

    assert_eq!(read(&media), b"photo");
    assert_eq!(read(&path), b"new");
}

#[test]
fn removes_app_files_only_inside_the_owned_folder() {
    let temp = tempdir().expect("temp dir");
    let owned = temp.path().join("runs");
    fs::create_dir_all(&owned).expect("create owned");
    let (log, media) = (owned.join("run.jsonl"), temp.path().join("photo.jpg"));
    write(&log, b"{}");
    write(&media, b"photo");

    assert!(remove_app_file(&media, &owned).is_err());
    assert!(remove_app_file(&owned.join("..").join("photo.jpg"), &owned).is_err());
    assert!(ensure_inside_dir(&owned.join("."), &owned).is_err());
    remove_app_file(&log, &owned).expect("remove log");

    assert!(!log.exists());
    assert_eq!(read(&media), b"photo");
}

#[cfg(unix)]
#[test]
fn refuses_to_remove_a_symlinked_app_file() {
    let temp = tempdir().expect("temp dir");
    let owned = temp.path().join("runs");
    fs::create_dir_all(&owned).expect("create owned");
    let media = temp.path().join("photo.jpg");
    write(&media, b"photo");
    std::os::unix::fs::symlink(&media, owned.join("run.jsonl")).expect("symlink");

    assert!(remove_app_file(&owned.join("run.jsonl"), &owned).is_err());
    assert_eq!(read(&media), b"photo");
}
