use std::path::Path;

use super::{placed_moves::plan_placed_moves, PlaceFolders};
use crate::{
    models::{MetadataResult, PlaceSuggestion, PlacedMove},
    path_safety,
    placements::PlacementStore,
    settings::SettingsStore,
};

/// The files earlier runs placed in the folders the accepted suggestions continue,
/// planned into the day layout the new files give those folders.
pub(crate) fn plan_continued_moves(
    settings: &SettingsStore,
    placements: &PlacementStore,
    selected_folder: &Path,
    suggestions: &[PlaceSuggestion],
    metadata_results: &[MetadataResult],
) -> Result<Vec<PlacedMove>, String> {
    path_safety::validate_selected_folder(selected_folder)?;
    let folders = PlaceFolders::new(suggestions, metadata_results)?;
    if folders.continued().is_empty() {
        return Ok(Vec::new());
    }
    let root = settings.place_folder_root(selected_folder)?;
    Ok(plan_placed_moves(&root, &folders, &placements.load()?))
}
