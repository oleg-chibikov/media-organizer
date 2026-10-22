use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    models::Placement,
    placements::{placed_files, PlacementStore},
    settings::SettingsStore,
};

/// The files earlier runs placed under the place folder root of `selected_folder`. Planning
/// keeps them in their folder. With no root or no readable registry, none are kept, so
/// plain planning still works.
pub(crate) fn placed_files_under(
    settings: &SettingsStore,
    placements: &PlacementStore,
    selected_folder: &Path,
) -> HashMap<PathBuf, Placement> {
    let Ok(root) = settings.place_folder_root(selected_folder) else {
        return HashMap::new();
    };
    match placements.load_existing() {
        Ok(records) => placed_files(&records, &root),
        Err(error) => {
            log::warn!("Failed to read placed suggestions: {error}");
            HashMap::new()
        }
    }
}
