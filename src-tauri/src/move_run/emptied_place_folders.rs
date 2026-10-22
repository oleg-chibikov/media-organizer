use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use super::request::MoveRunRequest;
use crate::{
    models::{MoveResult, MoveResultStatus, Placement, SuggestionKind},
    placements::top_folder,
    safe_fs,
};

/// Removes the day folders that placed files moved out of once they are empty. Each walk
/// starts at a source's folder and stops below its place folder, or its outing or trip
/// folder for files in no place. Those folders stay, and so do the year and the root.
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
        let stop = place_folder(&request.place_root, placement);
        if let Some(parent) = source.parent().filter(|parent| parent.starts_with(&stop)) {
            walks.insert((parent.to_path_buf(), stop));
        }
    }
    // Deepest first, so a parent is checked after every child that could empty it.
    let mut walks: Vec<_> = walks.into_iter().collect();
    walks.sort_by_key(|(start, _)| std::cmp::Reverse(start.components().count()));
    for (start, stop) in walks {
        safe_fs::remove_empty_parents(&start, &stop);
    }
}

fn place_folder(root: &Path, placement: &Placement) -> PathBuf {
    let folder = top_folder(
        root,
        placement.kind,
        placement.year,
        placement.country.as_deref(),
        &placement.title,
    );
    match (placement.kind, &placement.place) {
        (SuggestionKind::Trip, Some(place)) => folder.join(place),
        _ => folder,
    }
}
