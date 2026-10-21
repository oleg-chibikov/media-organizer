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

#[test]
fn store_update_applies_change_to_saved_settings() {
    let temp = tempdir().expect("temp dir should be created");
    let store = crate::settings::SettingsStore::in_app_data(temp.path());

    store
        .update(|current| crate::settings::set_force_processed_folder(current, true))
        .expect("settings should update");

    assert!(
        store
            .load()
            .expect("settings should load")
            .force_processed_folder
    );
}

#[test]
fn loads_settings_saved_before_rename_in_place_existed() {
    let temp = tempdir().expect("temp dir should be created");
    let path = destination_settings_path(temp.path());
    std::fs::create_dir_all(path.parent().expect("settings dir")).expect("dir should be created");
    let saved =
        r#"{"version":1,"root_media_folder":null,"force_processed_folder":true,"updated_at":null}"#;
    std::fs::write(&path, saved).expect("settings should be written");

    let loaded = load_destination_settings(temp.path()).expect("old settings should load");

    assert!(loaded.force_processed_folder);
    assert!(!loaded.rename_in_place);
}
