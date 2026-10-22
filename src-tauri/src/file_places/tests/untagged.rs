use super::Media;
use super::FRECHEN;

#[test]
fn a_file_without_gps_joins_the_place_of_the_same_device_within_3_hours() {
    let mut media = Media::default();
    media
        .burst("de", "2025-06-01 10:00:00", 1, 0, Some(FRECHEN))
        .burst("near", "2025-06-01 12:30:00", 1, 0, None)
        .burst("late", "2025-06-01 13:30:00", 1, 0, None);

    let places = media.detection(true).file_places;

    assert_eq!(places["near-0"], places["de-0"]);
    assert!(!places.contains_key("late-0"));
}

#[test]
fn a_file_without_gps_from_another_device_stays_out() {
    let mut media = Media::default();
    media
        .burst("de", "2025-06-01 10:00:00", 1, 0, Some(FRECHEN))
        .burst("other", "2025-06-01 10:10:00", 1, 0, None)
        .burst("unknown", "2025-06-01 10:20:00", 1, 0, None);
    media
        .with_device("other-0", Some("Canon EOS R6"))
        .with_device("unknown-0", None);

    let places = media.detection(true).file_places;

    assert!(!places.contains_key("other-0"));
    assert!(!places.contains_key("unknown-0"));
}
