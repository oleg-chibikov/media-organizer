use std::path::PathBuf;

use tempfile::tempdir;

use crate::settings::{
    set_root_media_folder,
    storage::{
        default_destination_settings, destination_settings_path, load_destination_settings,
        save_destination_settings,
    },
    DESTINATION_SETTINGS_VERSION,
};

#[test]
fn resolves_destination_settings_path_under_app_data_settings_dir() {
    let app_data_dir = PathBuf::from(r"C:\appdata\media-organizer");
    let path = destination_settings_path(&app_data_dir);
    let expected = app_data_dir
        .join("settings")
        .join("destination-settings.json");
    assert_eq!(path, expected);
}

#[test]
fn loads_default_settings_when_file_is_missing() {
    let temp = tempdir().expect("temp dir should be created");

    let loaded =
        load_destination_settings(temp.path()).expect("missing settings should load default");

    assert_eq!(loaded.version, DESTINATION_SETTINGS_VERSION);
    assert_eq!(loaded.root_media_folder, None);
    assert!(!loaded.force_processed_folder);
    assert!(loaded.updated_at.is_some());
}

#[test]
fn persists_and_reloads_destination_settings() {
    let temp = tempdir().expect("temp dir should be created");

    let updated = set_root_media_folder(
        &default_destination_settings(),
        Some(r"C:\Media\Library".to_string()),
    );
    let saved = save_destination_settings(temp.path(), &updated).expect("settings should be saved");
    let reloaded =
        load_destination_settings(temp.path()).expect("settings should be reloaded cleanly");

    assert_eq!(saved.version, DESTINATION_SETTINGS_VERSION);
    assert_eq!(reloaded.version, DESTINATION_SETTINGS_VERSION);
    assert_eq!(
        reloaded.root_media_folder,
        Some(r"C:\Media\Library".to_string())
    );
    assert!(!reloaded.force_processed_folder);
    assert!(reloaded.updated_at.is_some());
}
