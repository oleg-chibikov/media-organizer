use std::collections::BTreeMap;

use super::{continuation::adopt_place_names, PlacedSuggestion};
use crate::models::{ContinuedPlace, ContinuedPlacement, LiftedMonth, PlaceSuggestion};

/// Continues every placed folder of the suggestion's country in `records` at once. Places
/// of the same name merge their spans. At `year_level`, the month folders' files move up.
pub(super) fn continue_country(
    suggestion: &mut PlaceSuggestion,
    records: &[&PlacedSuggestion],
    year_level: bool,
) {
    let mut places: Vec<ContinuedPlace> = Vec::new();
    for placed in records.iter().flat_map(|record| &record.places) {
        match places
            .iter_mut()
            .find(|place| place.name.eq_ignore_ascii_case(&placed.name))
        {
            Some(place) => {
                place.start = place.start.clone().min(placed.start.clone());
                place.end = place.end.clone().max(placed.end.clone());
            }
            None => places.push(ContinuedPlace {
                name: placed.name.clone(),
                start: placed.start.clone(),
                end: placed.end.clone(),
            }),
        }
    }
    adopt_place_names(&mut suggestion.places, &places);
    let country = records
        .iter()
        .find_map(|record| record.country.clone())
        .or_else(|| suggestion.country.clone())
        .unwrap_or_default();
    suggestion.continues = Some(ContinuedPlacement {
        title: country,
        year: suggestion.year,
        start: records
            .iter()
            .map(|record| &record.start)
            .min()
            .cloned()
            .unwrap_or_default(),
        end: records
            .iter()
            .map(|record| &record.end)
            .max()
            .cloned()
            .unwrap_or_default(),
        places,
        country_wide: true,
        lifted: if year_level {
            lifted_months(records)
        } else {
            Vec::new()
        },
    });
}

/// File counts by month folder, in month order.
fn lifted_months(records: &[&PlacedSuggestion]) -> Vec<LiftedMonth> {
    let mut counts: BTreeMap<(u32, &str), u32> = BTreeMap::new();
    for record in records {
        let Some(month) = record.month_folder.as_deref() else {
            continue;
        };
        let files = record.root_files.len()
            + record
                .places
                .iter()
                .map(|place| place.files.len())
                .sum::<usize>();
        let count = counts
            .entry((month.parse().unwrap_or(0), month))
            .or_default();
        *count += u32::try_from(files).unwrap_or(u32::MAX);
    }
    counts
        .into_iter()
        .map(|((_, month), file_count)| LiftedMonth {
            month_folder: month.to_owned(),
            file_count,
        })
        .collect()
}
