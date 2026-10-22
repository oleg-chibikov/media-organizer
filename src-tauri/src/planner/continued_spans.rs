//! The dates and names a suggestion takes from the placed folder it continues.

use crate::models::{ContinuedPlacement, PlaceGroup, PlaceSuggestion, SuggestionKind};

/// A continued place also spans the files already placed in the folder of its name.
pub(super) fn place_span<'a>(
    group: &'a PlaceGroup,
    continued: Option<&'a ContinuedPlacement>,
) -> Vec<&'a String> {
    let placed = continued
        .into_iter()
        .flat_map(|placement| &placement.places)
        .find(|placed| placed.name == group.name);
    [&group.start, &group.end]
        .into_iter()
        .chain(
            placed
                .into_iter()
                .flat_map(|placed| [&placed.start, &placed.end]),
        )
        .collect()
}

pub(super) fn trip_span<'a>(
    suggestion: &'a PlaceSuggestion,
    continued: Option<&'a ContinuedPlacement>,
) -> Vec<&'a String> {
    [&suggestion.start, &suggestion.end]
        .into_iter()
        .chain(
            continued
                .into_iter()
                .flat_map(|placed| [&placed.start, &placed.end]),
        )
        .collect()
}

/// The placement a suggestion continues, unless the person renamed it in the review.
pub(super) fn continued_placement(suggestion: &PlaceSuggestion) -> Option<&ContinuedPlacement> {
    let name = match suggestion.kind {
        SuggestionKind::Trip => Some(&suggestion.title),
        SuggestionKind::Outing => suggestion.places.first().map(|place| &place.name),
    };
    suggestion
        .continues
        .as_ref()
        .filter(|placed| name == Some(&placed.title))
}
