use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::PlacedSuggestion;
use crate::models::Placement;

/// Each file the records under `root` hold, by its path, with the placement it is in.
pub(crate) fn placed_files(
    records: &[PlacedSuggestion],
    root: &Path,
) -> HashMap<PathBuf, Placement> {
    let mut placed = HashMap::new();
    for record in records.iter().filter(|record| record.is_in(root)) {
        let places = record
            .places
            .iter()
            .map(|place| (Some(place.name.as_str()), &place.files));
        for (place, files) in places.chain([(None, &record.root_files)]) {
            for file in files {
                placed.insert(root.join(&file.relative_path), record.placement(place));
            }
        }
    }
    placed
}
