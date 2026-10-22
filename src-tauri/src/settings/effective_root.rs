use std::path::{Path, PathBuf};

use super::custom_root_validation::validate_custom_root_media_folder;
use crate::models::{DestinationRootMode, DestinationSettings, EffectiveDestinationRoot};

const PROCESSED_FOLDER_NAME: &str = "Processed";

/// Renames in place when asked. Otherwise uses the configured custom root unless forced or
/// unset, then `<selected>/Processed`.
pub(super) fn resolve_effective_destination_root(
    selected_folder: &Path,
    settings: &DestinationSettings,
) -> Result<EffectiveDestinationRoot, String> {
    if settings.rename_in_place {
        return Ok(root(
            DestinationRootMode::InPlace,
            selected_folder,
            settings,
        ));
    }

    let processed_folder = selected_folder.join(PROCESSED_FOLDER_NAME);
    if settings.force_processed_folder {
        return Ok(root(
            DestinationRootMode::ProcessedFolder,
            &processed_folder,
            settings,
        ));
    }

    let Some(root_path) = configured_root(settings) else {
        return Ok(root(
            DestinationRootMode::ProcessedFolder,
            &processed_folder,
            settings,
        ));
    };

    let custom_root = PathBuf::from(root_path);
    if !custom_root.is_absolute() {
        return Err("Configured root Media folder path must be absolute.".to_string());
    }

    validate_custom_root_media_folder(&custom_root)?;

    Ok(root(
        DestinationRootMode::CustomRoot,
        &custom_root,
        settings,
    ))
}

/// Where accepted place folders go: the root a move uses with rename in place off.
pub(super) fn resolve_place_folder_root(
    selected_folder: &Path,
    settings: &DestinationSettings,
) -> Result<PathBuf, String> {
    let moving = DestinationSettings {
        rename_in_place: false,
        ..settings.clone()
    };
    let root = resolve_effective_destination_root(selected_folder, &moving)?;
    Ok(PathBuf::from(root.root_path))
}

fn configured_root(settings: &DestinationSettings) -> Option<&str> {
    settings
        .root_media_folder
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn root(
    mode: DestinationRootMode,
    root_path: &Path,
    settings: &DestinationSettings,
) -> EffectiveDestinationRoot {
    EffectiveDestinationRoot {
        mode,
        root_path: root_path.to_string_lossy().to_string(),
        custom_root_configured: configured_root(settings).is_some(),
        force_processed_folder: settings.force_processed_folder,
        warning: None,
    }
}
