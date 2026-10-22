use chrono::Utc;

use super::DESTINATION_SETTINGS_VERSION;
use crate::models::{DestinationSettings, HomeLocation, HomeSource};

/// Blank or whitespace-only roots clear the custom root.
pub(crate) fn set_root_media_folder(
    settings: &DestinationSettings,
    root_media_folder: Option<String>,
) -> DestinationSettings {
    let root_media_folder = root_media_folder
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty());
    stamped(DestinationSettings {
        root_media_folder,
        ..settings.clone()
    })
}

pub(crate) fn clear_root_media_folder(settings: &DestinationSettings) -> DestinationSettings {
    set_root_media_folder(settings, None)
}

pub(crate) fn set_force_processed_folder(
    settings: &DestinationSettings,
    force_processed_folder: bool,
) -> DestinationSettings {
    stamped(DestinationSettings {
        force_processed_folder,
        ..settings.clone()
    })
}

pub(crate) fn set_rename_in_place(
    settings: &DestinationSettings,
    rename_in_place: bool,
) -> DestinationSettings {
    stamped(DestinationSettings {
        rename_in_place,
        ..settings.clone()
    })
}

/// Saves the home the person picked, or clears it so a scan can learn one again.
pub(crate) fn set_home_location(
    settings: &DestinationSettings,
    home: Option<HomeLocation>,
) -> DestinationSettings {
    let home = home.map(|home| HomeLocation {
        source: HomeSource::Manual,
        ..home
    });
    stamped(DestinationSettings {
        home,
        ..settings.clone()
    })
}

pub(super) fn stamped(settings: DestinationSettings) -> DestinationSettings {
    DestinationSettings {
        version: DESTINATION_SETTINGS_VERSION,
        updated_at: Some(Utc::now().to_rfc3339()),
        ..settings
    }
}
