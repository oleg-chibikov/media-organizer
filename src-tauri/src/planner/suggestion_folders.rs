use std::collections::HashMap;

use chrono::{Datelike, NaiveDateTime};

use super::{
    place_dates::{day_layout_for, parse_date},
    place_folder::PlaceFolder,
};
use crate::{
    filename::validated_folder_name,
    models::{ContinuedPlacement, PlaceGroup, PlaceSuggestion, Placement, SuggestionKind},
};

/// One folder of an accepted suggestion and the files it takes.
pub(super) struct SuggestionFolder<'a> {
    pub(super) folder: PlaceFolder,
    pub(super) file_ids: &'a [String],
    pub(super) continued: bool,
}

/// The place folders of one suggestion, then a trip's root folder. A continued
/// suggestion keeps the placed year, and its trip root spans the placed files too.
pub(super) fn suggestion_folders<'a>(
    suggestion: &'a PlaceSuggestion,
    dates: &HashMap<&str, NaiveDateTime>,
) -> Result<Vec<SuggestionFolder<'a>>, String> {
    let continued = continued_placement(suggestion);
    let year = match continued {
        Some(placed) => placed.year,
        None => parse_date(&suggestion.start)?.year(),
    };
    let trip_title = match suggestion.kind {
        SuggestionKind::Trip => Some(validated_folder_name(&suggestion.title)?),
        SuggestionKind::Outing if suggestion.places.len() == 1 => None,
        SuggestionKind::Outing => {
            return Err(format!(
                "Outing \"{}\" needs exactly one place.",
                suggestion.title
            ))
        }
    };
    let base: Vec<String> = [year.to_string()]
        .into_iter()
        .chain(trip_title.clone())
        .collect();
    let placement = |title: String, place: Option<String>| Placement {
        kind: suggestion.kind,
        year,
        title,
        place,
    };
    let is_continued = continued.is_some();

    let mut folders = Vec::with_capacity(suggestion.places.len() + 1);
    for group in &suggestion.places {
        let name = validated_folder_name(&group.name)?;
        let title = trip_title.clone().unwrap_or_else(|| name.clone());
        let folder = PlaceFolder {
            parts: [base.clone(), vec![name.clone()]].concat(),
            day_layout: day_layout_for(&group.file_ids, &place_span(group, continued), dates)?,
            placement: placement(title, Some(name)),
        };
        folders.push(SuggestionFolder {
            folder,
            file_ids: &group.file_ids,
            continued: is_continued,
        });
    }
    if let Some(title) = trip_title {
        let span = trip_span(suggestion, continued);
        let folder = PlaceFolder {
            parts: base,
            day_layout: day_layout_for(&suggestion.root_file_ids, &span, dates)?,
            placement: placement(title, None),
        };
        folders.push(SuggestionFolder {
            folder,
            file_ids: &suggestion.root_file_ids,
            continued: is_continued,
        });
    }
    Ok(folders)
}

/// A continued place also spans the files already placed in the folder of its name.
fn place_span<'a>(
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

fn trip_span<'a>(
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
fn continued_placement(suggestion: &PlaceSuggestion) -> Option<&ContinuedPlacement> {
    let name = match suggestion.kind {
        SuggestionKind::Trip => Some(&suggestion.title),
        SuggestionKind::Outing => suggestion.places.first().map(|place| &place.name),
    };
    suggestion
        .continues
        .as_ref()
        .filter(|placed| name == Some(&placed.title))
}
