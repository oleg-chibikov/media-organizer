use super::fixtures::{daily, empty_store, saved_home, store_with_home, HOUSTON, LIVERPOOL_NSW};
use crate::{home_location::detect_file_places, models::HomeSource};

#[test]
fn a_saved_home_puts_a_trip_abroad_in_its_country() {
    let (_temp, store) = store_with_home("Sydney", (-33.8679, 151.2073), HomeSource::Manual);
    let (files, metadata) = daily("2025-10-01", 20, HOUSTON);

    let found = detect_file_places(&store, &files, &metadata)
        .expect("places")
        .file_places;

    assert!(found
        .values()
        .all(|place| place.country.as_deref() == Some("United States")));
    assert_eq!(
        saved_home(&store).map(|home| home.name),
        Some("Sydney".to_owned())
    );
}

#[test]
fn a_scan_with_60_days_at_home_saves_it_as_learned() {
    let (_temp, store) = empty_store();
    let (files, metadata) = daily("2025-01-01", 60, LIVERPOOL_NSW);

    detect_file_places(&store, &files, &metadata).expect("places");

    let home = saved_home(&store).expect("a learned home");
    assert_eq!(home.name, "Liverpool");
    assert_eq!(home.country, "Australia");
    assert_eq!(home.source, HomeSource::Learned);
}

#[test]
fn a_smaller_scan_infers_home_for_itself_and_saves_none() {
    let (_temp, store) = empty_store();
    let (mut files, mut metadata) = daily("2025-01-01", 59, LIVERPOOL_NSW);
    let (trip_files, trip_metadata) = daily("2025-03-10", 10, HOUSTON);
    files.extend(trip_files);
    metadata.extend(trip_metadata);

    let found = detect_file_places(&store, &files, &metadata)
        .expect("places")
        .file_places;

    let countries: std::collections::HashSet<Option<&str>> = found
        .values()
        .map(|place| place.country.as_deref())
        .collect();
    assert_eq!(countries, [None, Some("United States")].into(), "{found:?}");
    assert_eq!(saved_home(&store), None);
}

#[test]
fn a_large_scan_keeps_the_saved_home() {
    let (_temp, store) = store_with_home("Houston", HOUSTON, HomeSource::Manual);
    let (files, metadata) = daily("2025-01-01", 90, LIVERPOOL_NSW);

    detect_file_places(&store, &files, &metadata).expect("places");

    let home = saved_home(&store).expect("the saved home");
    assert_eq!(
        (home.name.as_str(), home.source),
        ("Houston", HomeSource::Manual)
    );
}
