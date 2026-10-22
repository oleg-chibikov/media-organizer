use std::path::Path;

use super::{matching::find_continued, PlacedSuggestion, PlacementStore};
use crate::{
    models::{ContinuedPlace, ContinuedPlacement, PlaceSuggestion},
    settings::SettingsStore,
};

/// Marks each suggestion that continues a placed trip or outing under the destination
/// folder root of `selected_folder`. It takes the record's title, year and place folder names.
pub(crate) fn continue_placed_suggestions(
    store: &PlacementStore,
    settings: &SettingsStore,
    selected_folder: &Path,
    mut suggestions: Vec<PlaceSuggestion>,
) -> Result<Vec<PlaceSuggestion>, String> {
    if suggestions.is_empty() {
        return Ok(suggestions);
    }
    let root = settings.place_folder_root(selected_folder)?;
    let records = store.load_existing()?;
    for suggestion in &mut suggestions {
        if let Some(record) = find_continued(&records, &root, suggestion) {
            continue_from(suggestion, record);
        }
    }
    Ok(suggestions)
}

fn continue_from(suggestion: &mut PlaceSuggestion, record: &PlacedSuggestion) {
    suggestion.title = record.title.clone();
    suggestion.year = record.year;
    suggestion.continues = Some(ContinuedPlacement {
        title: record.title.clone(),
        year: record.year,
        start: record.start.clone(),
        end: record.end.clone(),
        places: record
            .places
            .iter()
            .map(|placed| ContinuedPlace {
                name: placed.name.clone(),
                start: placed.start.clone(),
                end: placed.end.clone(),
            })
            .collect(),
    });
    for group in &mut suggestion.places {
        let placed = record
            .places
            .iter()
            .find(|placed| placed.name.eq_ignore_ascii_case(&group.name));
        if let Some(placed) = placed {
            group.name = placed.name.clone();
        }
    }
}
