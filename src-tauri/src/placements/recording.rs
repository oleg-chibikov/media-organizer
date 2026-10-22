use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use super::{
    lookup::{place_files, record_for},
    spans::{is_empty, refresh_spans},
    PlacedFile, PlacedSuggestion,
};
use crate::models::Placement;

/// One file a run moved into a place folder.
pub(crate) struct MovedIntoPlace {
    pub(crate) placement: Placement,
    pub(crate) source: PathBuf,
    pub(crate) destination: PathBuf,
    pub(crate) chosen_date: String,
}

/// Adds moved files to the record of their folder, creating it on first use. A file
/// moved to a new day folder of the same record leaves its old entry behind.
pub(crate) fn record_moves(
    records: &mut Vec<PlacedSuggestion>,
    root: &Path,
    moves: &[MovedIntoPlace],
) {
    let replaced: Vec<&Path> = moves
        .iter()
        .flat_map(|moved| [moved.source.as_path(), moved.destination.as_path()])
        .collect();
    forget_files(records, root, &replaced);
    for moved in moves {
        let Some(relative_path) = relative_to(root, &moved.destination) else {
            continue;
        };
        let file = PlacedFile {
            relative_path,
            chosen_date: moved.chosen_date.clone(),
        };
        let record = record_for(records, root, &moved.placement);
        match &moved.placement.place {
            Some(name) => place_files(record, name).push(file),
            None => record.root_files.push(file),
        }
    }
    records.iter_mut().for_each(refresh_spans);
}

/// Drops the files at `paths` from the records under `root`, and records left empty.
pub(crate) fn forget_files(records: &mut Vec<PlacedSuggestion>, root: &Path, paths: &[&Path]) {
    let gone: HashSet<String> = paths
        .iter()
        .filter_map(|path| relative_to(root, path))
        .collect();
    if gone.is_empty() {
        return;
    }
    let kept = |file: &PlacedFile| !gone.contains(&file.relative_path);
    for record in records.iter_mut().filter(|record| record.is_in(root)) {
        record.root_files.retain(kept);
        for place in &mut record.places {
            place.files.retain(kept);
        }
        record.places.retain(|place| !place.files.is_empty());
        refresh_spans(record);
    }
    records.retain(|record| !is_empty(record));
}

pub(super) fn relative_to(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let parts: Vec<String> = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy().to_string())
        .collect();
    (!parts.is_empty()).then(|| parts.join("/"))
}
