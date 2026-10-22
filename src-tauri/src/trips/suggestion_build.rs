//! Turns a grouped draft into the suggestion the frontend shows.

use chrono::Datelike;

use super::{
    day_layout::day_layout_for_span,
    draft::Draft,
    place_names::place_folder_names,
    place_table::PlaceTable,
    shots::{Shot, CHOSEN_DATE_FORMAT},
    suggestion_id::suggestion_id,
    trip_title::trip_title,
};
use crate::{
    filename::sanitize_for_windows_filename,
    models::{PlaceGroup, PlaceSuggestion, SuggestionKind},
};

/// `None` only for an empty draft, which grouping doesn't produce.
pub(super) fn build_suggestion(
    draft: &Draft,
    shots: &[Shot],
    places: &PlaceTable,
) -> Option<PlaceSuggestion> {
    let group_places: Vec<_> = draft.groups.iter().map(|group| group.place).collect();
    let names = place_folder_names(&group_places, places);
    let mut groups = Vec::with_capacity(draft.groups.len());
    for (group, name) in draft.groups.iter().zip(names) {
        let place_kind = places.get(group.place).kind.tag().to_owned();
        groups.push(place_group(name, place_kind, &group.shots, shots)?);
    }
    groups.sort_by(|a, b| a.start.cmp(&b.start));

    let members: Vec<usize> = draft
        .groups
        .iter()
        .flat_map(|group| &group.shots)
        .chain(&draft.root)
        .copied()
        .collect();
    let (first, last) = (
        &shots[*members.iter().min()?],
        &shots[*members.iter().max()?],
    );
    let title = match draft.kind {
        SuggestionKind::Trip => {
            let countries = members.iter().filter_map(|&index| {
                let place = shots[index].location?.resolved.place?;
                Some(places.get(place).country_name.as_str())
            });
            sanitize_for_windows_filename(&trip_title(countries))
        }
        SuggestionKind::Outing => groups.first()?.name.clone(),
    };
    let start = first.taken.format(CHOSEN_DATE_FORMAT).to_string();
    Some(PlaceSuggestion {
        id: suggestion_id(draft.kind, first.file_id, &start),
        kind: draft.kind,
        year: first.taken.year(),
        title,
        start,
        end: last.taken.format(CHOSEN_DATE_FORMAT).to_string(),
        file_count: u32::try_from(members.len()).unwrap_or(u32::MAX),
        places: groups,
        root_file_ids: file_ids(&draft.root, shots),
    })
}

fn place_group(
    name: String,
    place_kind: String,
    members: &[usize],
    shots: &[Shot],
) -> Option<PlaceGroup> {
    let first = shots[*members.iter().min()?].taken;
    let last = shots[*members.iter().max()?].taken;
    Some(PlaceGroup {
        name,
        place_kind,
        day_layout: day_layout_for_span(first.date(), last.date()),
        start: first.format(CHOSEN_DATE_FORMAT).to_string(),
        end: last.format(CHOSEN_DATE_FORMAT).to_string(),
        file_ids: file_ids(members, shots),
    })
}

/// File ids in time order. Shot indexes follow time, so sorting them is enough.
fn file_ids(members: &[usize], shots: &[Shot]) -> Vec<String> {
    let mut sorted = members.to_vec();
    sorted.sort_unstable();
    sorted
        .iter()
        .map(|&index| shots[index].file_id.to_owned())
        .collect()
}
