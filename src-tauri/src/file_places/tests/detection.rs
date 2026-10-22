use super::{Media, FRECHEN, LIVERPOOL_NSW, ROYAL_BOTANIC_GARDEN};
use crate::models::{DetectionStatus, FilePlace};

fn place(name: &str, country: Option<&str>) -> FilePlace {
    FilePlace {
        place: name.to_owned(),
        country: country.map(str::to_owned),
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
        place("Frechen", Some("Germany"))
    );
}

#[test]
fn a_place_in_home_country_has_no_country() {
    let mut media = Media::default();
    media.burst(
        "garden",
        "2025-06-01 10:00:00",
        1,
        0,
        Some(ROYAL_BOTANIC_GARDEN),
    );

    let detection = media.detection(true);

    assert_eq!(detection.file_places["garden-0"], place("Sydney", None));
}

#[test]
fn with_no_home_every_place_has_its_country() {
    let mut media = Media::default();
    media.burst("park", "2025-06-01 10:00:00", 1, 0, Some(LIVERPOOL_NSW));

    let found = &media.detection(false).file_places["park-0"];

    assert_eq!(found.country.as_deref(), Some("Australia"));
}

#[test]
fn files_without_gps_report_no_gps() {
    let mut media = Media::default();
    media.burst("bare", "2025-06-01 10:00:00", 6, 60, None);

    let detection = media.detection(true);

    assert_eq!(detection.status, DetectionStatus::NoGps);
    assert!(detection.file_places.is_empty());
}
