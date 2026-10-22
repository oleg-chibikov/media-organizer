//! One scan's files geocoded once, ready for home inference and grouping.

use super::{home, outings, place_table::PlaceTable, shots, suggestion_build, trip_runs, untagged};
use crate::{
    geo::Coordinates,
    models::{FileRecord, MetadataResult, PlaceSuggestion},
};

pub(crate) struct PlaceScan<'a> {
    shots: Vec<shots::Shot<'a>>,
    places: PlaceTable,
}

impl<'a> PlaceScan<'a> {
    pub(crate) fn new(files: &'a [FileRecord], metadata_results: &[MetadataResult]) -> Self {
        let mut places = PlaceTable::default();
        let shots = shots::collect_shots(files, metadata_results, &mut places);
        Self { shots, places }
    }

    /// The city with the most days, when it has enough of them to be home.
    pub(crate) fn home_city(&self) -> Option<home::HomeCity> {
        home::infer_home(&self.shots, &self.places)
    }

    /// Trips and outings in time order. With no `home` there are only outings.
    pub(crate) fn suggestions(&self, home: Option<Coordinates>) -> Vec<PlaceSuggestion> {
        let (shots, places) = (&self.shots, &self.places);
        let mut drafts = home
            .map(|home| trip_runs::find_trips(shots, home, places))
            .unwrap_or_default();
        let mut in_trip = vec![false; shots.len()];
        for trip in &drafts {
            let members = trip.groups.iter().flat_map(|group| &group.shots);
            for &index in members.chain(&trip.root) {
                in_trip[index] = true;
            }
        }
        drafts.extend(outings::find_outings(shots, &in_trip, places));
        untagged::attach_untagged(shots, &mut drafts);
        let mut suggestions: Vec<PlaceSuggestion> = drafts
            .iter()
            .filter_map(|draft| suggestion_build::build_suggestion(draft, shots, places))
            .collect();
        suggestions.sort_by(|a, b| a.start.cmp(&b.start));
        suggestions
    }
}
