use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::models::{DestinationRootMode, DestinationSettings, EffectiveDestinationRoot};

pub const DESTINATION_SETTINGS_VERSION: u32 = 1;

const SETTINGS_DIR_NAME: &str = "settings";
const SETTINGS_FILE_NAME: &str = "destination-settings.json";
const PROCESSED_FOLDER_NAME: &str = "Processed";

pub fn destination_settings_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(SETTINGS_DIR_NAME)
}

pub fn destination_settings_path(app_data_dir: &Path) -> PathBuf {
    destination_settings_dir(app_data_dir).join(SETTINGS_FILE_NAME)
}

pub fn default_destination_settings() -> DestinationSettings {
    DestinationSettings {
        version: DESTINATION_SETTINGS_VERSION,
        root_media_folder: None,
        force_processed_folder: false,
        updated_at: Some(Utc::now().to_rfc3339()),
    }
}

pub fn load_destination_settings(app_data_dir: &Path) -> Result<DestinationSettings, String> {
    let settings_path = destination_settings_path(app_data_dir);
    if !settings_path.exists() {
        return Ok(default_destination_settings());
    }

    let raw = fs::read_to_string(&settings_path).map_err(|error| {
        format!(
            "Failed to read destination settings file at {}: {error}",
            settings_path.display()
        )
    })?;

    let settings: DestinationSettings = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "Failed to parse destination settings file at {}: {error}",
            settings_path.display()
        )
    })?;

    if settings.version != DESTINATION_SETTINGS_VERSION {
        return Ok(default_destination_settings());
    }

    Ok(settings)
}

pub fn save_destination_settings(
    app_data_dir: &Path,
    settings: &DestinationSettings,
) -> Result<DestinationSettings, String> {
    let settings_path = destination_settings_path(app_data_dir);
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Failed to create destination settings directory at {}: {error}",
                parent.display()
            )
        })?;
    }

    let mut persisted = settings.clone();
    persisted.version = DESTINATION_SETTINGS_VERSION;
    persisted.updated_at = Some(Utc::now().to_rfc3339());

    let serialized = serde_json::to_string_pretty(&persisted)
        .map_err(|error| format!("Failed to serialize destination settings JSON: {error}"))?;

    fs::write(&settings_path, serialized).map_err(|error| {
        format!(
            "Failed to write destination settings file at {}: {error}",
            settings_path.display()
        )
    })?;

    Ok(persisted)
}

pub fn set_root_media_folder(
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

pub fn clear_root_media_folder(settings: &DestinationSettings) -> DestinationSettings {
    set_root_media_folder(settings, None)
}

pub fn set_force_processed_folder(
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

pub fn resolve_effective_destination_root(
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

    if let Some(root_path) = configured_root {
        let root = PathBuf::from(root_path);
        if !root.is_absolute() {
            return Err("Configured root Media folder path must be absolute.".to_string());
        }

        validate_custom_root_media_folder(&root)?;

        return Ok(EffectiveDestinationRoot {
            mode: DestinationRootMode::CustomRoot,
            root_path: root.to_string_lossy().to_string(),
            custom_root_configured: true,
            force_processed_folder: settings.force_processed_folder,
            warning: None,
        });
    }

    Ok(processed_folder_root(selected_folder, settings))
}

pub fn validate_custom_root_media_folder(root: &Path) -> Result<(), String> {
    if !root.exists() {
        return Err(format!(
            "Configured root Media folder does not exist: {}",
            root.display()
        ));
    }

    let metadata = fs::metadata(root).map_err(|error| {
        format!(
            "Configured root Media folder cannot be accessed: {} ({error})",
            root.display()
        )
    })?;

    if !metadata.is_dir() {
        return Err(format!(
            "Configured root Media folder is not a directory: {}",
            root.display()
        ));
    }

    fs::read_dir(root).map_err(|error| {
        format!(
            "Configured root Media folder is not readable: {} ({error})",
            root.display()
        )
    })?;

    Ok(())
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::{
        clear_root_media_folder, default_destination_settings, destination_settings_path,
        load_destination_settings, resolve_effective_destination_root, save_destination_settings,
        set_force_processed_folder, set_root_media_folder, DESTINATION_SETTINGS_VERSION,
    };
    use crate::models::DestinationRootMode;

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
        let saved =
            save_destination_settings(temp.path(), &updated).expect("settings should be saved");
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

        let resolved =
            resolve_effective_destination_root(&selected, &default_destination_settings())
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
}
