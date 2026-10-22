use super::{PlacedFile, PlacedSuggestion};

/// Sets each place and record span to the first and last date of its files.
/// `chosen_date` strings sort in time order.
pub(super) fn refresh_spans(record: &mut PlacedSuggestion) {
    for place in &mut record.places {
        if let Some((start, end)) = span_of(&place.files) {
            (place.start, place.end) = (start, end);
        }
    }
    let all_files: Vec<PlacedFile> = record
        .places
        .iter()
        .flat_map(|place| place.files.iter())
        .chain(&record.root_files)
        .cloned()
        .collect();
    if let Some((start, end)) = span_of(&all_files) {
        (record.start, record.end) = (start, end);
    }
}

pub(super) fn is_empty(record: &PlacedSuggestion) -> bool {
    record.root_files.is_empty() && record.places.iter().all(|place| place.files.is_empty())
}

fn span_of(files: &[PlacedFile]) -> Option<(String, String)> {
    let first = files.iter().map(|file| &file.chosen_date).min()?;
    let last = files.iter().map(|file| &file.chosen_date).max()?;
    Some((first.clone(), last.clone()))
}
