use crate::settings::{
    clear_root_media_folder, set_force_processed_folder, set_group_by_place, set_rename_in_place,
    set_root_media_folder, storage::default_destination_settings,
};

#[test]
fn clears_root_media_folder_and_updates_timestamp() {
    let settings = set_root_media_folder(
        &default_destination_settings(),
        Some(r"C:\Media\Library".to_string()),
    );
    let cleared = clear_root_media_folder(&settings);

    assert_eq!(cleared.root_media_folder, None);
    assert!(cleared.updated_at.is_some());
}

#[test]
fn updates_force_toggle_and_keeps_existing_root() {
    let settings = set_root_media_folder(
        &default_destination_settings(),
        Some(r"C:\Media\Library".to_string()),
    );
    let toggled = set_force_processed_folder(&settings, true);

    assert!(toggled.force_processed_folder);
    assert_eq!(
        toggled.root_media_folder,
        Some(r"C:\Media\Library".to_string())
    );
    assert!(toggled.updated_at.is_some());
}

#[test]
fn updates_rename_in_place_and_keeps_other_settings() {
    let with_root = set_root_media_folder(
        &default_destination_settings(),
        Some(r"C:\Media\Library".to_string()),
    );
    let toggled = set_rename_in_place(&set_force_processed_folder(&with_root, true), true);

    assert!(toggled.rename_in_place);
    assert!(toggled.force_processed_folder);
    assert_eq!(
        toggled.root_media_folder,
        Some(r"C:\Media\Library".to_string())
    );
}

#[test]
fn groups_by_place_by_default_and_turns_it_off() {
    let defaults = default_destination_settings();
    let off = set_group_by_place(&set_rename_in_place(&defaults, true), false);

    assert!(defaults.group_by_place);
    assert!(!off.group_by_place);
    assert!(off.rename_in_place);
}
