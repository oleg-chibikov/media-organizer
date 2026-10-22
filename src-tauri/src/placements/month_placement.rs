use std::path::Path;

use super::{
    continuation::continue_from, country_continuation::continue_country, matching::find_continued,
    MonthFolder, PlacedSuggestion,
};
use crate::models::PlaceSuggestion;

/// Files a suggestion found in a month folder scan. One at home stays in the month and
/// continues a placed folder of that month. One abroad joins its country's folders of
/// the year: inside the month while no other month holds that country, else at year level,
/// where the country's files from every month folder move up to.
pub(super) fn place_in_month(
    suggestion: &mut PlaceSuggestion,
    records: &[PlacedSuggestion],
    root: &Path,
    month: &MonthFolder,
) {
    suggestion.year = month.year;
    let in_month = |record: &&PlacedSuggestion| {
        record.year == month.year && record.month_folder.as_ref() == Some(&month.name)
    };
    let Some(country) = suggestion.country.clone() else {
        suggestion.month_folder = Some(month.name.clone());
        if let Some(record) = find_continued(records.iter().filter(in_month), root, suggestion) {
            continue_from(suggestion, record);
        }
        return;
    };
    let of_country: Vec<&PlacedSuggestion> = records
        .iter()
        .filter(|record| record.is_in(root) && record.year == month.year)
        .filter(|record| {
            record
                .country
                .as_deref()
                .is_some_and(|placed| placed.eq_ignore_ascii_case(&country))
        })
        .collect();
    let year_level = !of_country.iter().all(in_month);
    suggestion.month_folder = (!year_level).then(|| month.name.clone());
    if !of_country.is_empty() {
        continue_country(suggestion, &of_country, year_level);
    }
}
