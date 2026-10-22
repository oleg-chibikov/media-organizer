use super::{ids, Media, ROYAL_BOTANIC_GARDEN, SYDNEY};

const PERTH: (f64, f64) = (-31.95, 115.86);

#[test]
fn a_trip_in_home_country_gets_no_country_folder() {
    let mut media = Media::default();
    media.burst("perth", "2025-06-01 10:00:00", 6, 60, Some(PERTH));

    let trips = media.detect(Some(SYDNEY));

    assert_eq!(trips.len(), 1, "{trips:#?}");
    assert_eq!(trips[0].country, None);
    assert_eq!(trips[0].title, "Australia");
}

#[test]
fn files_with_no_gps_in_a_trip_root_go_to_the_main_country() {
    let mut media = Media::default();
    media
        .burst("de", "2025-06-01 10:00:00", 6, 60, Some(super::FRECHEN))
        .burst("lost", "2025-06-02 10:00:00", 1, 0, None)
        .burst("again", "2025-06-03 10:00:00", 5, 60, Some(super::FRECHEN));

    let trips = media.detect(Some(SYDNEY));

    assert_eq!(trips.len(), 1, "{trips:#?}");
    assert_eq!(trips[0].country.as_deref(), Some("Germany"));
    assert!(trips[0].root_file_ids.contains(&ids("lost", 1)[0]));
}

#[test]
fn an_outing_near_home_gets_no_country_folder() {
    let mut media = Media::default();
    media.burst(
        "garden",
        "2025-03-01 10:00:00",
        5,
        10,
        Some(ROYAL_BOTANIC_GARDEN),
    );

    assert_eq!(media.detect(Some(SYDNEY))[0].country, None);
}

#[test]
fn with_an_unknown_home_an_outing_gets_its_country_folder() {
    let mut media = Media::default();
    media.burst(
        "garden",
        "2025-03-01 10:00:00",
        5,
        10,
        Some(ROYAL_BOTANIC_GARDEN),
    );

    let outings = media.detect(None);

    assert_eq!(outings[0].country.as_deref(), Some("Australia"));
}
