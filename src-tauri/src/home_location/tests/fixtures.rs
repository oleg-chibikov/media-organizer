use chrono::{NaiveDateTime, TimeDelta};
use tempfile::{tempdir, TempDir};

use crate::{
    file_places::CHOSEN_DATE_FORMAT,
    models::{FileKind, FileRecord, GpsPoint, HomeLocation, HomeSource, MetadataResult},
    settings::{self, SettingsStore},
    test_support::media::{dated_metadata, file_record},
};

pub(super) const HOUSTON: (f64, f64) = (29.76, -95.37);
/// Resolves to a park, with Liverpool as its nearest city.
pub(super) const LIVERPOOL_NSW: (f64, f64) = (-33.92, 150.92);

/// One photo a day for `days` days from `first_day` at `point`.
pub(super) fn daily(
    first_day: &str,
    days: usize,
    point: (f64, f64),
) -> (Vec<FileRecord>, Vec<MetadataResult>) {
    let start = NaiveDateTime::parse_from_str(&format!("{first_day} 12:00:00"), CHOSEN_DATE_FORMAT)
        .expect("a date");
    (0..days)
        .map(|day| {
            let id = format!("{first_day}-{day}");
            let taken = start + TimeDelta::days(day as i64);
            let mut metadata = dated_metadata(&id, &taken.format(CHOSEN_DATE_FORMAT).to_string());
            metadata.gps = Some(GpsPoint {
                latitude: point.0,
                longitude: point.1,
            });
            (file_record(&id, "jpg", FileKind::Photo), metadata)
        })
        .unzip()
}

pub(super) fn empty_store() -> (TempDir, SettingsStore) {
    let temp = tempdir().expect("temp dir");
    let store = SettingsStore::in_app_data(temp.path());
    (temp, store)
}

pub(super) fn store_with_home(
    name: &str,
    point: (f64, f64),
    source: HomeSource,
) -> (TempDir, SettingsStore) {
    let (temp, store) = empty_store();
    let home = HomeLocation {
        name: name.to_owned(),
        country: "Australia".to_owned(),
        latitude: point.0,
        longitude: point.1,
        source,
    };
    store
        .update(|current| settings::replace_home_location(current, home))
        .expect("home saved");
    (temp, store)
}

pub(super) fn saved_home(store: &SettingsStore) -> Option<HomeLocation> {
    store.load().expect("settings load").home
}
