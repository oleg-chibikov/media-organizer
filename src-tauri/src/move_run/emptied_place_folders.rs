use std::{
    collections::{HashMap, HashSet},
    path::{Component, Path, PathBuf},
};

use super::request::MoveRunRequest;
use crate::{
    models::{MoveResult, MoveResultStatus, Placement},
    placements::place_folder,
    safe_fs,
};

/// Removes the day folders that placed files moved out of once they are empty. Each walk
/// starts at a source's folder and stops below its place folder, or its outing or trip
/// folder for files in no place. Those folders stay, and so do the year and the root.
/// A file lifted from a month folder to the year walks up to that month folder, so its
/// emptied country folder goes too.
pub(super) fn remove_emptied_place_folders(request: &MoveRunRequest, results: &[MoveResult]) {
    let placements: HashMap<&str, &Placement> = request
        .batch
        .plans
        .iter()
        .filter_map(|plan| Some((plan.file_id.as_str(), plan.placement.as_ref()?)))
        .collect();
    let mut walks: HashSet<(PathBuf, PathBuf)> = HashSet::new();
    for result in results {
        if !matches!(result.result_status, MoveResultStatus::Moved) {
            continue;
        }
        let Some(placement) = placements.get(result.file_id.as_str()) else {
            continue;
        };
        let source = Path::new(&result.attempted_source_path);
        let (Some(parent), Some(stop)) = (
            source.parent(),
            walk_stop(&request.place_root, placement, source),
        ) else {
            continue;
        };
        walks.insert((parent.to_path_buf(), stop));
    }
    // Deepest first, so a parent is checked after every child that could empty it.
    let mut walks: Vec<_> = walks.into_iter().collect();
    walks.sort_by_key(|(start, _)| std::cmp::Reverse(start.components().count()));
    for (start, stop) in walks {
        safe_fs::remove_empty_parents(&start, &stop);
    }
}

/// The place folder holding `source`, or the month folder a year level file came from.
fn walk_stop(root: &Path, placement: &Placement, source: &Path) -> Option<PathBuf> {
    let stop = place_folder(root, placement);
    if source.starts_with(&stop) {
        return Some(stop);
    }
    if placement.month_folder.is_some() {
        return None;
    }
    let year_folder = root.join(placement.year.to_string());
    let Component::Normal(month) = source
        .strip_prefix(&year_folder)
        .ok()?
        .components()
        .next()?
    else {
        return None;
    };
    let lifted_from = Placement {
        month_folder: Some(month.to_string_lossy().to_string()),
        ..placement.clone()
    };
    source
        .starts_with(place_folder(root, &lifted_from))
        .then(|| year_folder.join(month))
}
