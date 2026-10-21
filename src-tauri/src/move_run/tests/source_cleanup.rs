use std::fs;

use tempfile::tempdir;

use crate::move_run::source_cleanup::cleanup_empty_source_directories;

#[test]
fn removes_empty_source_directory_chain_after_moves() {
    let temp = tempdir().expect("temp dir");
    let selected_root = temp.path().join("todo");
    let nested = selected_root.join("a").join("b");
    fs::create_dir_all(&nested).expect("create nested source dirs");

    cleanup_empty_source_directories(&selected_root, std::slice::from_ref(&nested));

    assert!(!nested.exists());
    assert!(!selected_root.join("a").exists());
    assert!(!selected_root.exists());
}

#[test]
fn stops_pruning_when_parent_contains_files() {
    let temp = tempdir().expect("temp dir");
    let selected_root = temp.path().join("todo");
    let parent = selected_root.join("a");
    let nested = parent.join("b");
    fs::create_dir_all(&nested).expect("create nested source dirs");
    fs::write(parent.join("keep.txt"), b"keep").expect("write retained file");

    cleanup_empty_source_directories(&selected_root, std::slice::from_ref(&nested));

    assert!(!nested.exists());
    assert!(parent.exists());
}

#[test]
fn removes_selected_root_when_processing_leaves_it_empty() {
    let temp = tempdir().expect("temp dir");
    let selected_root = temp.path().join("todo");
    fs::create_dir_all(&selected_root).expect("create selected root");

    cleanup_empty_source_directories(&selected_root, std::slice::from_ref(&selected_root));

    assert!(!selected_root.exists());
}

#[test]
fn keeps_selected_root_when_it_still_contains_files() {
    let temp = tempdir().expect("temp dir");
    let selected_root = temp.path().join("todo");
    fs::create_dir_all(&selected_root).expect("create selected root");
    fs::write(selected_root.join("keep.txt"), b"keep").expect("write retained file");

    cleanup_empty_source_directories(&selected_root, std::slice::from_ref(&selected_root));

    assert!(selected_root.exists());
}
