use std::{collections::HashSet, path::Path};

use super::{placed_file::plan_placed_file, PlaceFolders};
use crate::{models::PlacedMove, placements::PlacedSuggestion};

/// Plans the files earlier runs placed in a continued folder into its new day folders,
/// and the files of a country wide continuation into the folders at its level.
/// A file already in the right day folder, or no longer on disk, is left out.
pub(super) fn plan_placed_moves(
    root: &Path,
    folders: &PlaceFolders,
    records: &[PlacedSuggestion],
) -> Result<Vec<PlacedMove>, String> {
    let mut reserved_paths = HashSet::new();
    let mut moves = Vec::new();
    for folder in folders.continued() {
        let placement = &folder.placement;
        let Some(record) = records.iter().find(|record| record.holds(root, placement)) else {
            continue;
        };
        for file in record.files_in(placement.place.as_deref()) {
            moves.extend(plan_placed_file(root, folder, file, &mut reserved_paths));
        }
    }
    for scope in folders.country_scopes() {
        for record in records.iter().filter(|record| scope.takes(root, record)) {
            let places = record.places.iter().map(|place| Some(place.name.as_str()));
            for place in places.chain([None]) {
                let folder = scope.folder_for(record, place, folders.accepted())?;
                for file in record.files_in(place) {
                    moves.extend(plan_placed_file(root, &folder, file, &mut reserved_paths));
                }
            }
        }
    }
    Ok(moves)
}
