use std::path::{Path, PathBuf};

use crate::models::{Placement, SuggestionKind};

/// `<root>/<year>[/<month>]/<title>`, with an outing abroad inside `<country>`. A trip
/// abroad is titled after its country already.
pub(crate) fn top_folder(root: &Path, placement: &Placement) -> PathBuf {
    let year_folder = root.join(placement.year.to_string());
    let level = match &placement.month_folder {
        Some(month) => year_folder.join(month),
        None => year_folder,
    };
    match (placement.kind, &placement.country) {
        (SuggestionKind::Outing, Some(country)) => level.join(country).join(&placement.title),
        _ => level.join(&placement.title),
    }
}

/// The folder a placement's day folders go in: a trip's place, or the trip or outing folder.
pub(crate) fn place_folder(root: &Path, placement: &Placement) -> PathBuf {
    let folder = top_folder(root, placement);
    match (placement.kind, &placement.place) {
        (SuggestionKind::Trip, Some(place)) => folder.join(place),
        _ => folder,
    }
}
