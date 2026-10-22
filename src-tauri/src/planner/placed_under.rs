use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    models::{FileRecord, Placement},
    placements::{placed_files, relative_to, PlacementStore},
    settings::SettingsStore,
};

/// The `files` earlier runs placed under the place folder root of `selected_folder`.
/// Planning keeps them in their folder. With no root or no readable registry, none are
/// kept, so plain planning still works. With no file under the root, the registry isn't read.
pub(crate) fn placed_files_under(
    settings: &SettingsStore,
    placements: &PlacementStore,
    selected_folder: &Path,
    files: &[FileRecord],
) -> HashMap<PathBuf, Placement> {
    let Ok(root) = settings.place_folder_root(selected_folder) else {
        return HashMap::new();
    };
    let sources = || files.iter().map(|file| Path::new(&file.source_path));
    if relative_to(&root, sources()).is_empty() {
        return HashMap::new();
    }
    match placements.load_existing() {
        Ok(records) => placed_files(&records, &root, sources()),
        Err(error) => {
            log::warn!("Failed to read placed suggestions: {error}");
            HashMap::new()
        }
    }
}
