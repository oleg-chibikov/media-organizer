use std::collections::HashMap;

use chrono::{Datelike, NaiveDateTime};

use super::{
    continued_spans::{continued_placement, place_span, trip_span},
    place_dates::{day_layout_for, parse_date},
    place_folder::PlaceFolder,
};
use crate::{
    filename::validated_folder_name,
    models::{PlaceSuggestion, Placement, SuggestionKind},
};

/// One folder of an accepted suggestion and the files it takes.
pub(super) struct SuggestionFolder<'a> {
    pub(super) folder: PlaceFolder,
    pub(super) file_ids: &'a [String],
}

/// The place folders of one suggestion, then a trip's root folder. A continued
/// suggestion keeps the placed year, and its trip root spans the placed files too.
/// A suggestion in a month folder takes the year of that folder.
pub(super) fn suggestion_folders<'a>(
    suggestion: &'a PlaceSuggestion,
    dates: &HashMap<&str, NaiveDateTime>,
) -> Result<Vec<SuggestionFolder<'a>>, String> {
    let continued = continued_placement(suggestion);
    let year = match (continued, &suggestion.month_folder) {
        (Some(placed), _) => placed.year,
        (None, Some(_)) => suggestion.year,
        (None, None) => parse_date(&suggestion.start)?.year(),
    };
    // A trip abroad is filed under its country, whatever its title says.
    let trip_folder = suggestion.country.as_ref().unwrap_or(&suggestion.title);
    let trip_title = match suggestion.kind {
        SuggestionKind::Trip => Some(validated_folder_name(trip_folder)?),
        SuggestionKind::Outing if suggestion.places.len() == 1 => None,
        SuggestionKind::Outing => {
            return Err(format!(
                "Outing \"{}\" needs exactly one place.",
                suggestion.title
            ))
        }
    };
    let country = suggestion
        .country
        .as_deref()
        .map(validated_folder_name)
        .transpose()?;
    let month_folder = suggestion
        .month_folder
        .as_deref()
        .map(validated_folder_name)
        .transpose()?;
    let placement = |title: String, place: Option<String>| Placement {
        kind: suggestion.kind,
        year,
        title,
        place,
        country: country.clone(),
        month_folder: month_folder.clone(),
    };

    let mut folders = Vec::with_capacity(suggestion.places.len() + 1);
    for group in &suggestion.places {
        let name = validated_folder_name(&group.name)?;
        let title = trip_title.clone().unwrap_or_else(|| name.clone());
        let folder = PlaceFolder {
            day_layout: day_layout_for(&group.file_ids, &place_span(group, continued), dates)?,
            placement: placement(title, Some(name)),
        };
        folders.push(SuggestionFolder {
            folder,
            file_ids: &group.file_ids,
        });
    }
    if let Some(title) = trip_title {
        let span = trip_span(suggestion, continued);
        let folder = PlaceFolder {
            day_layout: day_layout_for(&suggestion.root_file_ids, &span, dates)?,
            placement: placement(title, None),
        };
        folders.push(SuggestionFolder {
            folder,
            file_ids: &suggestion.root_file_ids,
        });
    }
    Ok(folders)
}
