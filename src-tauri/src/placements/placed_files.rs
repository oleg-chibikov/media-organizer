use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use super::PlacedSuggestion;
use crate::models::Placement;

/// The placement of each of `sources` that the records under `root` hold, by path.
// A plan batch holds a few files and the registry can hold tens of thousands, so only
// the wanted files are joined and cloned.
pub(crate) fn placed_files<'a>(
    records: &[PlacedSuggestion],
    root: &Path,
    sources: impl IntoIterator<Item = &'a Path>,
) -> HashMap<PathBuf, Placement> {
    let wanted = relative_to(root, sources);
    let mut placed = HashMap::new();
    if wanted.is_empty() {
        return placed;
    }
    for record in records.iter().filter(|record| record.is_in(root)) {
        let places = record
            .places
            .iter()
            .map(|place| (Some(place.name.as_str()), &place.files));
        for (place, files) in places.chain([(None, &record.root_files)]) {
            for file in files {
                let relative = Path::new(&file.relative_path);
                if wanted.contains(relative) {
                    placed.insert(root.join(relative), record.placement(place));
                }
            }
        }
    }
    placed
}

/// The paths of `sources` under `root`, relative to it. Empty when none is under it.
pub(crate) fn relative_to<'a>(
    root: &Path,
    sources: impl IntoIterator<Item = &'a Path>,
) -> HashSet<&'a Path> {
    sources
        .into_iter()
        .filter_map(|source| source.strip_prefix(root).ok())
        .collect()
}
