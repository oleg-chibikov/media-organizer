//! Splits a suggestion by country folder: one part per country outside home's
//! country, and one part with no country folder for home's country.

use std::collections::HashMap;

use super::{draft::Draft, place_table::PlaceTable, shots::Shot};
use crate::models::SuggestionKind;

/// A draft whose files all go under one country folder, or under none.
pub(super) struct CountryPart {
    pub(super) country: Option<String>,
    pub(super) draft: Draft,
}

/// With no known home every country is foreign. Trip root files go to the country
/// they were shot in, or the trip's main country when they have no GPS.
pub(super) fn split_by_country(
    draft: Draft,
    shots: &[Shot],
    places: &PlaceTable,
    home_country: Option<&str>,
) -> Vec<CountryPart> {
    let folder_for = |country: &str| {
        let foreign = home_country.is_none_or(|home| !home.eq_ignore_ascii_case(country));
        foreign.then(|| country.to_owned())
    };
    let shot_country = |index: usize| {
        let place = shots[index].location?.resolved.place?;
        Some(places.get(place).country_name.as_str())
    };
    let main_country = main_country(&draft, shots, places);
    let mut parts: Vec<CountryPart> = Vec::new();
    for group in draft.groups {
        let country = folder_for(&places.get(group.place).country_name);
        part_for(&mut parts, draft.kind, country).groups.push(group);
    }
    for shot in draft.root {
        let country = shot_country(shot).or(main_country.as_deref());
        let country = country.and_then(folder_for);
        part_for(&mut parts, draft.kind, country).root.push(shot);
    }
    parts
}

fn part_for(
    parts: &mut Vec<CountryPart>,
    kind: SuggestionKind,
    country: Option<String>,
) -> &mut Draft {
    let index = match parts.iter().position(|part| part.country == country) {
        Some(index) => index,
        None => {
            parts.push(CountryPart {
                country,
                draft: Draft {
                    kind,
                    groups: Vec::new(),
                    root: Vec::new(),
                    anchors: Vec::new(),
                },
            });
            parts.len() - 1
        }
    };
    &mut parts[index].draft
}

/// The country with the most geotagged files.
fn main_country(draft: &Draft, shots: &[Shot], places: &PlaceTable) -> Option<String> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let members = draft.groups.iter().flat_map(|group| &group.shots);
    for &index in members.chain(&draft.root) {
        if let Some(place) = shots[index]
            .location
            .and_then(|location| location.resolved.place)
        {
            *counts
                .entry(places.get(place).country_name.as_str())
                .or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(a.0)))
        .map(|(country, _)| country.to_owned())
}
