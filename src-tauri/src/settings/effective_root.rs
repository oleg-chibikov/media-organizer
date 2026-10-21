use std::path::{Path, PathBuf};

use super::custom_root_validation::validate_custom_root_media_folder;
use crate::models::{DestinationRootMode, DestinationSettings, EffectiveDestinationRoot};

const PROCESSED_FOLDER_NAME: &str = "Processed";

/// Uses the configured custom root unless forced or unset, then `<selected>/Processed`.
pub(super) fn resolve_effective_destination_root(
    selected_folder: &Path,
    settings: &DestinationSettings,
) -> Result<EffectiveDestinationRoot, String> {
    if settings.force_processed_folder {
        return Ok(processed_folder_root(selected_folder, settings));
    }

    let configured_root = settings
        .root_media_folder
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());

    let Some(root_path) = configured_root else {
        return Ok(processed_folder_root(selected_folder, settings));
    };

    let root = PathBuf::from(root_path);
    if !root.is_absolute() {
        return Err("Configured root Media folder path must be absolute.".to_string());
    }

    validate_custom_root_media_folder(&root)?;

    Ok(EffectiveDestinationRoot {
        mode: DestinationRootMode::CustomRoot,
        root_path: root.to_string_lossy().to_string(),
        custom_root_configured: true,
        force_processed_folder: settings.force_processed_folder,
        warning: None,
    })
}

fn processed_folder_root(
    selected_folder: &Path,
    settings: &DestinationSettings,
) -> EffectiveDestinationRoot {
    let custom_root_configured = settings
        .root_media_folder
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);

    EffectiveDestinationRoot {
        mode: DestinationRootMode::ProcessedFolder,
        root_path: selected_folder
            .join(PROCESSED_FOLDER_NAME)
            .to_string_lossy()
            .to_string(),
        custom_root_configured,
        force_processed_folder: settings.force_processed_folder,
        warning: None,
    }
}
