use super::{Media, FRECHEN, ROYAL_BOTANIC_GARDEN, SYDNEY};
use crate::models::DetectionStatus;

#[test]
fn found_suggestions_report_found_and_label_every_geotagged_file() {
    let mut media = Media::default();
    media
        .burst("de", "2025-06-01 10:00:00", 6, 60, Some(FRECHEN))
        .burst("bare", "2025-06-01 11:30:00", 1, 0, None);

    let detection = media.detection(Some(SYDNEY));

    assert_eq!(detection.status, DetectionStatus::Found);
    assert_eq!(detection.file_places.len(), 6);
    assert_eq!(detection.file_places["de-0"], "Frechen, Germany");
    assert!(!detection.file_places.contains_key("bare-0"));
}

#[test]
fn files_without_gps_report_no_gps() {
    let mut media = Media::default();
    media.burst("bare", "2025-06-01 10:00:00", 6, 60, None);

    assert_eq!(media.detection(Some(SYDNEY)).status, DetectionStatus::NoGps);
}

#[test]
fn an_unknown_home_with_no_outing_reports_home_unknown() {
    let mut media = Media::default();
    media.burst("de", "2025-06-01 10:00:00", 6, 60, Some(FRECHEN));

    assert_eq!(media.detection(None).status, DetectionStatus::HomeUnknown);
}

#[test]
fn files_near_home_report_the_home_city() {
    let mut media = Media::default();
    media.burst(
        "local",
        "2025-06-01 10:00:00",
        4,
        60,
        Some(ROYAL_BOTANIC_GARDEN),
    );

    let detection = media.detection(Some(SYDNEY));

    let home = "Sydney".to_owned();
    assert_eq!(detection.status, DetectionStatus::AllNearHome { home });
    assert!(!detection.file_places["local-0"].contains(','));
}

#[test]
fn a_few_files_away_or_at_a_place_report_too_few() {
    let mut media = Media::default();
    media
        .burst("de", "2025-06-01 10:00:00", 2, 60, Some(FRECHEN))
        .burst(
            "garden",
            "2025-06-05 10:00:00",
            2,
            10,
            Some(ROYAL_BOTANIC_GARDEN),
        );

    let status = media.detection(Some(SYDNEY)).status;

    assert_eq!(status, DetectionStatus::TooFewGeotagged);
}
