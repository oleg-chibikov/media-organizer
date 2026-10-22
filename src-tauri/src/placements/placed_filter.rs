use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use chrono::NaiveDateTime;

use crate::{
    models::{FileRecord, MetadataResult, PlaceSuggestion, Placement},
    trips::{day_layout_for_span, CHOSEN_DATE_FORMAT},
};

/// Takes the files earlier runs placed out of the suggestions, so a rescan doesn't
/// suggest their folder again. A suggestion left with new files keeps their dates and can
/// continue the placed folder. One left with none is dropped.
pub(super) fn without_placed_files(
    suggestions: Vec<PlaceSuggestion>,
    placed: &HashMap<PathBuf, Placement>,
    files: &[FileRecord],
    metadata_results: &[MetadataResult],
) -> Vec<PlaceSuggestion> {
    let placed_ids: HashSet<&str> = files
        .iter()
        .filter(|file| placed.contains_key(Path::new(&file.source_path)))
        .map(|file| file.id.as_str())
        .collect();
    if placed_ids.is_empty() {
        return suggestions;
    }
    let dates: HashMap<&str, &str> = metadata_results
        .iter()
        .filter_map(|result| Some((result.file_id.as_str(), result.chosen_date.as_deref()?)))
        .collect();
    suggestions
        .into_iter()
        .filter_map(|suggestion| without(suggestion, &placed_ids, &dates))
        .collect()
}

fn without(
    mut suggestion: PlaceSuggestion,
    placed: &HashSet<&str>,
    dates: &HashMap<&str, &str>,
) -> Option<PlaceSuggestion> {
    let unplaced = |ids: &mut Vec<String>| ids.retain(|id| !placed.contains(id.as_str()));
    for group in &mut suggestion.places {
        unplaced(&mut group.file_ids);
        if let Some((start, end)) = span(&group.file_ids, dates) {
            group.day_layout = day_layout_for_span(parse(&start)?, parse(&end)?);
            (group.start, group.end) = (start, end);
        }
    }
    suggestion.places.retain(|group| !group.file_ids.is_empty());
    unplaced(&mut suggestion.root_file_ids);
    let ids: Vec<String> = suggestion
        .places
        .iter()
        .flat_map(|group| group.file_ids.iter().cloned())
        .chain(suggestion.root_file_ids.iter().cloned())
        .collect();
    let (start, end) = span(&ids, dates)?;
    suggestion.year = start.get(..4)?.parse().ok()?;
    suggestion.file_count = u32::try_from(ids.len()).unwrap_or(u32::MAX);
    (suggestion.start, suggestion.end) = (start, end);
    Some(suggestion)
}

/// The first and last date of `ids`. `chosen_date` strings sort in time order.
fn span(ids: &[String], dates: &HashMap<&str, &str>) -> Option<(String, String)> {
    let known = || ids.iter().filter_map(|id| dates.get(id.as_str()));
    Some((known().min()?.to_string(), known().max()?.to_string()))
}

fn parse(date: &str) -> Option<chrono::NaiveDate> {
    NaiveDateTime::parse_from_str(date, CHOSEN_DATE_FORMAT)
        .ok()
        .map(|date| date.date())
}
