use chrono::{NaiveDateTime, TimeDelta};
use tempfile::{tempdir, TempDir};

use super::{detect_place_suggestions, search_home_cities};
use crate::{
    models::{
        FileKind, FileRecord, GpsPoint, HomeLocation, HomeSource, MetadataResult, SuggestionKind,
    },
    settings::{self, SettingsStore},
    test_support::media::{dated_metadata, file_record},
    trips::CHOSEN_DATE_FORMAT,
};

const HOUSTON: (f64, f64) = (29.76, -95.37);
/// Resolves to a park, with Liverpool as its nearest city.
const LIVERPOOL_NSW: (f64, f64) = (-33.92, 150.92);

/// One photo a day for `days` days from `first_day` at `point`.
fn daily(
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

fn store_with_home(name: &str, point: (f64, f64)) -> (TempDir, SettingsStore) {
    let temp = tempdir().expect("temp dir");
    let store = SettingsStore::in_app_data(temp.path());
    let home = HomeLocation {
        name: name.to_owned(),
        country: "Australia".to_owned(),
        latitude: point.0,
        longitude: point.1,
        source: HomeSource::Manual,
    };
    store
        .update(|current| settings::set_home_location(current, Some(home)))
        .expect("home saved");
    (temp, store)
}

fn saved_home(store: &SettingsStore) -> Option<HomeLocation> {
    store.load().expect("settings load").home
}

#[test]
fn a_saved_home_finds_the_trip_in_one_month_folder() {
    let (_temp, store) = store_with_home("Sydney", (-33.8679, 151.2073));
    let (files, metadata) = daily("2025-10-01", 20, HOUSTON);

    let found = detect_place_suggestions(&store, &files, &metadata).expect("suggestions");

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].kind, SuggestionKind::Trip);
    assert_eq!(found[0].title, "United States");
    assert_eq!(
        saved_home(&store).map(|home| home.name),
        Some("Sydney".to_owned())
    );
}

#[test]
fn a_scan_with_60_days_at_home_saves_it_as_learned() {
    let temp = tempdir().expect("temp dir");
    let store = SettingsStore::in_app_data(temp.path());
    let (files, metadata) = daily("2025-01-01", 60, LIVERPOOL_NSW);

    detect_place_suggestions(&store, &files, &metadata).expect("suggestions");

    let home = saved_home(&store).expect("a learned home");
    assert_eq!(home.name, "Liverpool");
    assert_eq!(home.country, "Australia");
    assert_eq!(home.source, HomeSource::Learned);
}

#[test]
fn a_smaller_scan_infers_home_for_itself_and_saves_none() {
    let temp = tempdir().expect("temp dir");
    let store = SettingsStore::in_app_data(temp.path());
    let (mut files, mut metadata) = daily("2025-01-01", 59, LIVERPOOL_NSW);
    let (trip_files, trip_metadata) = daily("2025-03-10", 10, HOUSTON);
    files.extend(trip_files);
    metadata.extend(trip_metadata);

    let found = detect_place_suggestions(&store, &files, &metadata).expect("suggestions");

    let trips: Vec<&str> = found
        .iter()
        .filter(|suggestion| suggestion.kind == SuggestionKind::Trip)
        .map(|suggestion| suggestion.title.as_str())
        .collect();
    assert_eq!(trips, ["United States"], "{found:?}");
    assert_eq!(saved_home(&store), None);
}

#[test]
fn a_large_scan_keeps_the_saved_home() {
    let (_temp, store) = store_with_home("Houston", HOUSTON);
    let (files, metadata) = daily("2025-01-01", 90, LIVERPOOL_NSW);

    detect_place_suggestions(&store, &files, &metadata).expect("suggestions");

    let home = saved_home(&store).expect("the saved home");
    assert_eq!(
        (home.name.as_str(), home.source),
        ("Houston", HomeSource::Manual)
    );
}

#[test]
fn home_search_returns_up_to_ten_candidates_with_regions() {
    let found = search_home_cities("zür");

    assert_eq!(found[0].name, "Zuerich");
    assert_eq!(found[0].country, "Switzerland");
    assert!(found[0].region.is_some());
    assert_eq!(search_home_cities("a").len(), 10);
}
