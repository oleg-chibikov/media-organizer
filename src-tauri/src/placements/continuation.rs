use std::path::Path;

use super::{
    matching::find_continued, month_placement::place_in_month, MonthFolder, PlacedSuggestion,
    PlacementStore,
};
use crate::{
    models::{ContinuedPlace, ContinuedPlacement, PlaceGroup, PlaceSuggestion},
    settings::SettingsStore,
};

/// Marks each suggestion that continues a placed trip or outing under the destination
/// folder root of `selected_folder`. It takes the record's title, year and place folder names.
/// A scan of a month folder files its suggestions in that month, see `place_in_month`.
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
    let month = MonthFolder::of(&root, selected_folder);
    for suggestion in &mut suggestions {
        if let Some(month) = &month {
            place_in_month(suggestion, &records, &root, month);
            continue;
        }
        let year_level = records
            .iter()
            .filter(|record| record.month_folder.is_none());
        if let Some(record) = find_continued(year_level, &root, suggestion) {
            continue_from(suggestion, record);
        }
    }
    Ok(suggestions)
}

pub(super) fn continue_from(suggestion: &mut PlaceSuggestion, record: &PlacedSuggestion) {
    suggestion.title = record.title.clone();
    suggestion.year = record.year;
    let places: Vec<ContinuedPlace> = record
        .places
        .iter()
        .map(|placed| ContinuedPlace {
            name: placed.name.clone(),
            start: placed.start.clone(),
            end: placed.end.clone(),
        })
        .collect();
    adopt_place_names(&mut suggestion.places, &places);
    suggestion.continues = Some(ContinuedPlacement {
        title: record.title.clone(),
        year: record.year,
        start: record.start.clone(),
        end: record.end.clone(),
        places,
        country_wide: false,
        lifted: Vec::new(),
    });
}

/// A found place takes the folder name of the placed place it matches, ignoring case.
pub(super) fn adopt_place_names(groups: &mut [PlaceGroup], placed: &[ContinuedPlace]) {
    for group in groups {
        if let Some(placed) = placed
            .iter()
            .find(|placed| placed.name.eq_ignore_ascii_case(&group.name))
        {
            group.name = placed.name.clone();
        }
    }
}
