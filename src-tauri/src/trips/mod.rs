//! Suggests place folders: trips far from home and outings to notable places.
//!
//! Pure grouping over file dates and GPS points. It makes no filesystem calls.
//! Geocoding loads the place table on first use, so call it off the UI thread.

mod day_layout;
mod draft;
mod home;
mod outings;
mod place_names;
mod place_table;
mod shots;
mod suggestion_build;
mod suggestion_id;
mod thresholds;
mod trip_places;
mod trip_runs;
mod trip_title;
mod untagged;

#[cfg(test)]
mod tests;

use crate::{
    geo::Coordinates,
    models::{FileRecord, MetadataResult, PlaceSuggestion},
};
pub(crate) use day_layout::day_layout_for_span;
pub(crate) use shots::CHOSEN_DATE_FORMAT;

/// Trips and outings in time order. With `home` unset, home is inferred
/// from the files; with no clear home there are only outings.
pub(crate) fn detect_place_suggestions(
    files: &[FileRecord],
    metadata_results: &[MetadataResult],
    home: Option<Coordinates>,
) -> Vec<PlaceSuggestion> {
    let mut places = place_table::PlaceTable::default();
    let shots = shots::collect_shots(files, metadata_results, &mut places);
    let home = home.or_else(|| home::infer_home(&shots, &places));
    let mut drafts = home
        .map(|home| trip_runs::find_trips(&shots, home, &places))
        .unwrap_or_default();
    let mut in_trip = vec![false; shots.len()];
    for trip in &drafts {
        let members = trip.groups.iter().flat_map(|group| &group.shots);
        for &index in members.chain(&trip.root) {
            in_trip[index] = true;
        }
    }
    drafts.extend(outings::find_outings(&shots, &in_trip, &places));
    untagged::attach_untagged(&shots, &mut drafts);
    let mut suggestions: Vec<PlaceSuggestion> = drafts
        .iter()
        .filter_map(|draft| suggestion_build::build_suggestion(draft, &shots, &places))
        .collect();
    suggestions.sort_by(|a, b| a.start.cmp(&b.start));
    suggestions
}
