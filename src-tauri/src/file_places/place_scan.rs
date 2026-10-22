//! One scan's files geocoded once, ready for home inference and place folders.

use super::{
    folder_names::folder_places, home, place_table::PlaceTable, shots, untagged::untagged_places,
};
use crate::models::{DetectionStatus, FileRecord, MetadataResult, PlaceDetection};

pub(crate) struct PlaceScan<'a> {
    shots: Vec<shots::Shot<'a>>,
    places: PlaceTable,
}

impl<'a> PlaceScan<'a> {
    pub(crate) fn new(files: &'a [FileRecord], metadata_results: &'a [MetadataResult]) -> Self {
        let mut places = PlaceTable::default();
        let shots = shots::collect_shots(files, metadata_results, &mut places);
        Self { shots, places }
    }

    /// The city with the most days, when it has enough of them to be home.
    pub(crate) fn home_city(&self) -> Option<home::HomeCity> {
        home::infer_home(&self.shots, &self.places)
    }

    /// The place folder of each geotagged file, and of each file with no GPS taken by the
    /// same device close in time.
    pub(crate) fn detect(&self, home: Option<&home::Home>) -> PlaceDetection {
        let home_country = home.map(|home| home.country.as_str());
        let mut file_places = folder_places(&self.shots, &self.places, home_country);
        let status = if file_places.is_empty() {
            DetectionStatus::NoGps
        } else {
            DetectionStatus::Found
        };
        let untagged = untagged_places(&self.shots, &file_places);
        file_places.extend(untagged);
        PlaceDetection {
            file_places,
            status,
        }
    }
}
