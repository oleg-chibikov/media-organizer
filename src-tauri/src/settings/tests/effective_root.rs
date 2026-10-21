use std::fs;

use tempfile::tempdir;

use crate::{
    models::DestinationRootMode,
    settings::{
        effective_root::resolve_effective_destination_root, set_force_processed_folder,
        set_rename_in_place, set_root_media_folder, storage::default_destination_settings,
    },
};

#[test]
fn resolves_custom_root_when_configured_and_valid() {
    let temp = tempdir().expect("temp dir should be created");
    let selected = temp.path().join("todo");
    let custom_root = temp.path().join("media-root");
    fs::create_dir_all(&selected).expect("selected folder should be created");
    fs::create_dir_all(&custom_root).expect("custom root should be created");

    let settings = set_root_media_folder(
        &default_destination_settings(),
        Some(custom_root.to_string_lossy().to_string()),
    );
    let resolved = resolve_effective_destination_root(&selected, &settings)
        .expect("custom root should resolve");

    assert_eq!(resolved.mode, DestinationRootMode::CustomRoot);
    assert_eq!(
        resolved.root_path,
        custom_root.to_string_lossy().to_string()
    );
    assert!(resolved.custom_root_configured);
    assert!(!resolved.force_processed_folder);
}

#[test]
fn resolves_processed_fallback_when_custom_root_is_not_configured() {
    let temp = tempdir().expect("temp dir should be created");
    let selected = temp.path().join("todo");
    fs::create_dir_all(&selected).expect("selected folder should be created");

    let resolved = resolve_effective_destination_root(&selected, &default_destination_settings())
        .expect("fallback processed root should resolve");

    assert_eq!(resolved.mode, DestinationRootMode::ProcessedFolder);
    assert_eq!(
        resolved.root_path,
        selected.join("Processed").to_string_lossy().to_string()
    );
    assert!(!resolved.custom_root_configured);
    assert!(!resolved.force_processed_folder);
}

#[test]
fn resolves_processed_when_force_toggle_is_enabled() {
    let temp = tempdir().expect("temp dir should be created");
    let selected = temp.path().join("todo");
    let custom_root = temp.path().join("media-root");
    fs::create_dir_all(&selected).expect("selected folder should be created");
    fs::create_dir_all(&custom_root).expect("custom root should be created");

    let with_root = set_root_media_folder(
        &default_destination_settings(),
        Some(custom_root.to_string_lossy().to_string()),
    );
    let forced = set_force_processed_folder(&with_root, true);
    let resolved = resolve_effective_destination_root(&selected, &forced)
        .expect("forced processed should resolve");

    assert_eq!(resolved.mode, DestinationRootMode::ProcessedFolder);
    assert_eq!(
        resolved.root_path,
        selected.join("Processed").to_string_lossy().to_string()
    );
    assert!(resolved.custom_root_configured);
    assert!(resolved.force_processed_folder);
}

#[test]
fn rejects_invalid_custom_root_during_resolution() {
    let temp = tempdir().expect("temp dir should be created");
    let selected = temp.path().join("todo");
    fs::create_dir_all(&selected).expect("selected folder should be created");
    let missing_root = temp.path().join("missing-root");

    let settings = set_root_media_folder(
        &default_destination_settings(),
        Some(missing_root.to_string_lossy().to_string()),
    );
    let error = resolve_effective_destination_root(&selected, &settings)
        .expect_err("missing custom root should fail resolution");

    assert!(error.contains("does not exist"));
}

#[test]
fn resolves_selected_folder_when_renaming_in_place() {
    let temp = tempdir().expect("temp dir should be created");
    let selected = temp.path().join("todo");
    let custom_root = temp.path().join("media-root");
    fs::create_dir_all(&selected).expect("selected folder should be created");
    fs::create_dir_all(&custom_root).expect("custom root should be created");

    let with_root = set_root_media_folder(
        &default_destination_settings(),
        Some(custom_root.to_string_lossy().to_string()),
    );
    let in_place = set_rename_in_place(&set_force_processed_folder(&with_root, true), true);
    let resolved = resolve_effective_destination_root(&selected, &in_place)
        .expect("in place root should resolve");

    assert_eq!(resolved.mode, DestinationRootMode::InPlace);
    assert_eq!(resolved.root_path, selected.to_string_lossy().to_string());
    assert!(resolved.custom_root_configured);
}
