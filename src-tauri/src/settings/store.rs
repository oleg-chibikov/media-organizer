use std::{
    path::{Path, PathBuf},
    sync::{Mutex, PoisonError},
};

use super::{
    effective_root::{resolve_effective_destination_root, resolve_place_folder_root},
    storage::{load_destination_settings, save_destination_settings},
};
use crate::models::{DestinationSettings, EffectiveDestinationRoot};

static SETTINGS_LOCK: Mutex<()> = Mutex::new(());

/// Destination settings persisted under `<appData>/settings`.
#[derive(Debug, Clone)]
pub(crate) struct SettingsStore {
    app_data_dir: PathBuf,
}

impl SettingsStore {
    pub(crate) fn in_app_data(app_data_dir: &Path) -> Self {
        Self {
            app_data_dir: app_data_dir.to_path_buf(),
        }
    }

    pub(crate) fn load(&self) -> Result<DestinationSettings, String> {
        load_destination_settings(&self.app_data_dir)
    }

    /// Applies `change` to the saved settings; concurrent updates run one at a time.
    pub(crate) fn update(
        &self,
        change: impl FnOnce(&DestinationSettings) -> DestinationSettings,
    ) -> Result<DestinationSettings, String> {
        let _guard = SETTINGS_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let current = self.load()?;
        save_destination_settings(&self.app_data_dir, &change(&current))
    }

    pub(crate) fn effective_destination_root(
        &self,
        selected_folder: &Path,
    ) -> Result<EffectiveDestinationRoot, String> {
        resolve_effective_destination_root(selected_folder, &self.load()?)
    }

    /// The root place folders go under, which differs from the effective root in rename in
    /// place mode.
    pub(crate) fn place_folder_root(&self, selected_folder: &Path) -> Result<PathBuf, String> {
        resolve_place_folder_root(selected_folder, &self.load()?)
    }
}
