use super::fixtures::{daily, saved_home, store_with_home, HOUSTON, LIVERPOOL_NSW};
use crate::{home_location::detect_file_places, models::HomeSource};

const SYDNEY: (f64, f64) = (-33.8679, 151.2073);

#[test]
fn a_scan_with_60_days_at_home_replaces_a_time_zone_guess() {
    let (_temp, store) = store_with_home("Sydney", SYDNEY, HomeSource::TimeZone);
    let (files, metadata) = daily("2025-01-01", 60, LIVERPOOL_NSW);

    detect_file_places(&store, &files, &metadata).expect("places");

    let home = saved_home(&store).expect("a learned home");
    assert_eq!(
        (home.name.as_str(), home.source),
        ("Liverpool", HomeSource::Learned)
    );
}

#[test]
fn a_scan_with_60_days_at_home_keeps_a_device_home() {
    let (_temp, store) = store_with_home("Sydney", SYDNEY, HomeSource::Device);
    let (files, metadata) = daily("2025-01-01", 60, LIVERPOOL_NSW);

    detect_file_places(&store, &files, &metadata).expect("places");

    let home = saved_home(&store).expect("the device home");
    assert_eq!(
        (home.name.as_str(), home.source),
        ("Sydney", HomeSource::Device)
    );
}

#[test]
fn a_time_zone_guess_puts_a_trip_abroad_in_its_country() {
    let (_temp, store) = store_with_home("Sydney", SYDNEY, HomeSource::TimeZone);
    let (files, metadata) = daily("2025-10-01", 20, HOUSTON);

    let found = detect_file_places(&store, &files, &metadata)
        .expect("places")
        .file_places;

    assert!(found
        .values()
        .all(|place| place.country_folder() == Some("United States")));
    assert_eq!(
        saved_home(&store).map(|home| home.source),
        Some(HomeSource::TimeZone)
    );
}
