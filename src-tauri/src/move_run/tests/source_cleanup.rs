use std::{fs, path::Path};

use tempfile::tempdir;

use crate::{
    move_run::remove_empty_source_folders,
    settings::{set_rename_in_place, set_root_media_folder, SettingsStore},
};

fn settings_in(app_data: &Path) -> SettingsStore {
    SettingsStore::in_app_data(app_data)
}

#[test]
fn removes_the_selected_folder_when_only_empty_folders_are_left() {
    let temp = tempdir().expect("temp dir");
    let selected = temp.path().join("todo");
    fs::create_dir_all(selected.join("2024").join("trip")).expect("create nested");
    fs::create_dir_all(selected.join("never-had-media")).expect("create empty sibling");

    let removed = remove_empty_source_folders(&settings_in(temp.path()), &selected);

    assert_eq!(removed, Ok(true));
    assert!(!selected.exists());
}

#[test]
fn keeps_files_left_behind_and_the_processed_folder() {
    let temp = tempdir().expect("temp dir");
    let selected = temp.path().join("todo");
    let processed = selected.join("Processed").join("2026");
    fs::create_dir_all(&processed).expect("create processed");
    fs::create_dir_all(selected.join("a").join("b")).expect("create empty chain");
    fs::create_dir_all(selected.join("notes")).expect("create notes");
    fs::write(selected.join("notes").join("readme.txt"), b"keep").expect("write note");

    let removed = remove_empty_source_folders(&settings_in(temp.path()), &selected);

    assert_eq!(removed, Ok(false));
    assert!(processed.exists());
    assert!(selected.join("notes").join("readme.txt").exists());
    assert!(!selected.join("a").exists());
}

#[test]
fn leaves_folders_alone_when_renaming_in_place() {
    let temp = tempdir().expect("temp dir");
    let selected = temp.path().join("todo");
    fs::create_dir_all(selected.join("empty")).expect("create empty");
    let settings = settings_in(temp.path());
    settings
        .update(|current| set_rename_in_place(current, true))
        .expect("turn on rename in place");

    assert_eq!(remove_empty_source_folders(&settings, &selected), Ok(false));
    assert!(selected.join("empty").exists());
}

#[test]
fn removes_the_selected_folder_inside_a_custom_root() {
    let temp = tempdir().expect("temp dir");
    let media = temp.path().join("Media");
    let selected = media.join("todo");
    fs::create_dir_all(selected.join("empty")).expect("create empty");
    let settings = settings_in(temp.path());
    let media_path = Some(media.to_string_lossy().to_string());
    settings
        .update(|current| set_root_media_folder(current, media_path))
        .expect("set custom root");

    assert_eq!(remove_empty_source_folders(&settings, &selected), Ok(true));
    assert!(!selected.exists());
    assert!(media.exists());
}

#[test]
fn does_nothing_when_the_selected_folder_is_already_gone() {
    let temp = tempdir().expect("temp dir");
    let missing = temp.path().join("missing");

    assert_eq!(
        remove_empty_source_folders(&settings_in(temp.path()), &missing),
        Ok(false)
    );
}
