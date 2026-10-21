use chrono::Utc;

use super::DESTINATION_SETTINGS_VERSION;
use crate::models::DestinationSettings;

/// Blank or whitespace-only roots clear the custom root.
pub(crate) fn set_root_media_folder(
    settings: &DestinationSettings,
    root_media_folder: Option<String>,
) -> DestinationSettings {
    DestinationSettings {
        version: DESTINATION_SETTINGS_VERSION,
        root_media_folder: root_media_folder.and_then(|path| {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }),
        force_processed_folder: settings.force_processed_folder,
        updated_at: Some(Utc::now().to_rfc3339()),
    }
}

pub(crate) fn clear_root_media_folder(settings: &DestinationSettings) -> DestinationSettings {
    set_root_media_folder(settings, None)
}

pub(crate) fn set_force_processed_folder(
    settings: &DestinationSettings,
    force_processed_folder: bool,
) -> DestinationSettings {
    DestinationSettings {
        version: DESTINATION_SETTINGS_VERSION,
        root_media_folder: settings.root_media_folder.clone(),
        force_processed_folder,
        updated_at: Some(Utc::now().to_rfc3339()),
    }
}
