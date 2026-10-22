use super::{Media, FRECHEN, LIVERPOOL_NSW, ROYAL_BOTANIC_GARDEN};
use crate::models::{DetectionStatus, FilePlace, HomeMatch};

fn place(name: &str, country: &str, home: HomeMatch) -> FilePlace {
    FilePlace {
        place: name.to_owned(),
        country: country.to_owned(),
        home,
    }
}

#[test]
fn every_geotagged_file_gets_its_place_with_a_foreign_country() {
    let mut media = Media::default();
    media.burst("de", "2025-06-01 10:00:00", 1, 0, Some(FRECHEN));

    let detection = media.detection(true);

    assert_eq!(detection.status, DetectionStatus::Found);
    assert_eq!(
        detection.file_places["de-0"],
        place("Frechen", "Germany", HomeMatch::Away)
    );
}

#[test]
fn a_place_in_home_country_matches_home_country() {
    let mut media = Media::default();
    media.burst(
        "garden",
        "2025-06-01 10:00:00",
        1,
        0,
        Some(ROYAL_BOTANIC_GARDEN),
    );

    let detection = media.detection(true);

    assert_eq!(
        detection.file_places["garden-0"],
        place("Sydney", "Australia", HomeMatch::Country)
    );
}

#[test]
fn a_place_in_home_city_matches_home_city() {
    let mut media = Media::default();
    media.burst("park", "2025-06-01 10:00:00", 1, 0, Some(LIVERPOOL_NSW));

    let found = &media.detection(true).file_places["park-0"];

    assert_eq!(
        (found.country.as_str(), found.home),
        ("Australia", HomeMatch::City)
    );
}

#[test]
fn with_no_home_every_place_is_away() {
    let mut media = Media::default();
    media.burst("park", "2025-06-01 10:00:00", 1, 0, Some(LIVERPOOL_NSW));

    let found = &media.detection(false).file_places["park-0"];

    assert_eq!(found.home, HomeMatch::Away);
}

#[test]
fn files_without_gps_report_no_gps() {
    let mut media = Media::default();
    media.burst("bare", "2025-06-01 10:00:00", 6, 60, None);

    let detection = media.detection(true);

    assert_eq!(detection.status, DetectionStatus::NoGps);
    assert!(detection.file_places.is_empty());
}
