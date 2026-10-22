use std::path::Path;

use chrono::{NaiveDateTime, TimeDelta};

use super::PlacedSuggestion;
use crate::{
    models::{PlaceSuggestion, SuggestionKind},
    trips::{CHOSEN_DATE_FORMAT, TRIP_MAX_GAP},
};

/// The record `suggestion` continues: same root and kind, at most `TRIP_MAX_GAP`
/// apart, and for an outing the same place. The closest record wins.
pub(super) fn find_continued<'a>(
    records: &'a [PlacedSuggestion],
    root: &Path,
    suggestion: &PlaceSuggestion,
) -> Option<&'a PlacedSuggestion> {
    let (start, end) = (parse(&suggestion.start)?, parse(&suggestion.end)?);
    records
        .iter()
        .filter(|record| record.is_in(root) && record.kind == suggestion.kind)
        .filter(|record| same_outing_place(record, suggestion))
        .filter_map(|record| {
            let gap = gap_between((start, end), (parse(&record.start)?, parse(&record.end)?));
            (gap <= TRIP_MAX_GAP).then_some((gap, record))
        })
        .min_by_key(|(gap, _)| *gap)
        .map(|(_, record)| record)
}

/// Zero when the spans overlap.
fn gap_between(
    (start, end): (NaiveDateTime, NaiveDateTime),
    (placed_start, placed_end): (NaiveDateTime, NaiveDateTime),
) -> TimeDelta {
    (start - placed_end)
        .max(placed_start - end)
        .max(TimeDelta::zero())
}

fn same_outing_place(record: &PlacedSuggestion, suggestion: &PlaceSuggestion) -> bool {
    match suggestion.kind {
        SuggestionKind::Trip => true,
        SuggestionKind::Outing => suggestion
            .places
            .first()
            .is_some_and(|place| place.name.eq_ignore_ascii_case(&record.title)),
    }
}

fn parse(date: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(date, CHOSEN_DATE_FORMAT).ok()
}
