mod detection;
mod home;
mod untagged;

use chrono::{NaiveDateTime, TimeDelta};

use crate::{
    file_places::{shots::CHOSEN_DATE_FORMAT, Home, PlaceScan},
    models::{FileKind, FileRecord, GpsPoint, MetadataResult, PlaceDetection},
    test_support::media::{dated_metadata, file_record},
};

/// Points that resolve to a city with no point of interest around them.
const FRECHEN: (f64, f64) = (50.9, 6.8);
const CASOLE_D_ELSA: (f64, f64) = (43.3, 11.0);
const REGGIO_EMILIA: (f64, f64) = (44.7, 10.6);
const HOFSTETTEN: (f64, f64) = (48.0, 11.0);
/// Resolves to a park, with Liverpool as its nearest city.
const LIVERPOOL_NSW: (f64, f64) = (-33.92, 150.92);
const ROYAL_BOTANIC_GARDEN: (f64, f64) = (-33.8642, 151.2166);

#[derive(Default)]
struct Media {
    files: Vec<FileRecord>,
    metadata: Vec<MetadataResult>,
}

impl Media {
    /// Adds `count` files from `start`, one every `step_minutes`, all from one iPhone.
    fn burst(
        &mut self,
        prefix: &str,
        start: &str,
        count: usize,
        step_minutes: i64,
        gps: Option<(f64, f64)>,
    ) -> &mut Self {
        let start = NaiveDateTime::parse_from_str(start, CHOSEN_DATE_FORMAT).expect("a date");
        for step in 0..count {
            let id = format!("{prefix}-{step}");
            let taken = start + TimeDelta::minutes(step_minutes * step as i64);
            let mut metadata = dated_metadata(&id, &taken.format(CHOSEN_DATE_FORMAT).to_string());
            metadata.gps = gps.map(|(latitude, longitude)| GpsPoint {
                latitude,
                longitude,
            });
            self.files.push(file_record(&id, "jpg", FileKind::Photo));
            self.metadata.push(metadata);
        }
        self
    }

    fn with_device(&mut self, file_id: &str, device: Option<&str>) -> &mut Self {
        let metadata = self
            .metadata
            .iter_mut()
            .find(|metadata| metadata.file_id == file_id)
            .expect("a file");
        metadata.device_name = device.map(str::to_owned);
        self
    }

    /// Detection with home in Liverpool, Australia, or none.
    fn detection(&self, home_in_australia: bool) -> PlaceDetection {
        let scan = PlaceScan::new(&self.files, &self.metadata);
        let home = home_in_australia.then(|| Home {
            city: "Liverpool".to_owned(),
            country: "Australia".to_owned(),
        });
        scan.detect(home.as_ref())
    }
}
