use crate::models::{DayLayout, PlaceGroup, PlaceSuggestion, SuggestionKind};

pub(super) fn group(name: &str, start: &str, end: &str, file_ids: &[&str]) -> PlaceGroup {
    PlaceGroup {
        name: name.to_owned(),
        place_kind: "city".to_owned(),
        // Planning recomputes this from the dates.
        day_layout: DayLayout::Single,
        start: start.to_owned(),
        end: end.to_owned(),
        file_ids: file_ids.iter().map(|id| (*id).to_owned()).collect(),
    }
}

pub(super) fn suggestion(
    kind: SuggestionKind,
    title: &str,
    places: Vec<PlaceGroup>,
) -> PlaceSuggestion {
    let start = places[0].start.clone();
    let end = places[places.len() - 1].end.clone();
    PlaceSuggestion {
        id: "test".to_owned(),
        kind,
        year: 2025,
        title: title.to_owned(),
        start,
        end,
        file_count: 0,
        places,
        root_file_ids: Vec::new(),
    }
}
