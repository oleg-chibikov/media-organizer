use super::{ids, Media, CASOLE_D_ELSA, REGGIO_EMILIA, SYDNEY};
use crate::models::{DayLayout, PlaceSuggestion};

const TRASTEVERE_CHURCH: (f64, f64) = (41.89, 12.47);
const BATHS_OF_DIOCLETIAN: (f64, f64) = (41.9028, 12.4964);

/// Rome points of interest with too few files each, a full city and a small city.
fn italy_trip() -> PlaceSuggestion {
    let mut media = Media::default();
    media
        .burst(
            "church",
            "2025-05-01 10:00:00",
            4,
            30,
            Some(TRASTEVERE_CHURCH),
        )
        .burst(
            "baths",
            "2025-05-01 13:00:00",
            2,
            30,
            Some(BATHS_OF_DIOCLETIAN),
        )
        .burst("casole", "2025-05-02 10:00:00", 6, 30, Some(CASOLE_D_ELSA))
        .burst("near", "2025-05-02 14:00:00", 1, 0, None)
        .burst("far", "2025-05-02 20:00:00", 1, 0, None)
        .burst("reggio", "2025-05-03 10:00:00", 3, 30, Some(REGGIO_EMILIA))
        .burst("outside", "2025-05-10 10:00:00", 1, 0, None);
    let mut suggestions = media.detect(Some(SYDNEY));
    assert_eq!(suggestions.len(), 1, "{suggestions:#?}");
    suggestions.remove(0)
}

#[test]
fn small_points_of_interest_fall_back_to_their_city() {
    let trip = italy_trip();

    let rome = &trip.places[0];
    assert_eq!(rome.name, "Rome");
    assert_eq!(rome.place_kind, "city");
    assert_eq!(rome.day_layout, DayLayout::Single);
    assert_eq!(rome.file_ids, [ids("church", 4), ids("baths", 2)].concat());
    assert_eq!(trip.title, "Italy");
}

#[test]
fn a_small_city_falls_back_to_the_trip_root() {
    let trip = italy_trip();

    assert_eq!(trip.places.len(), 2);
    assert_eq!(
        trip.root_file_ids,
        [ids("far", 1), ids("reggio", 3)].concat()
    );
}

#[test]
fn files_without_gps_join_the_place_taken_within_three_hours() {
    let trip = italy_trip();

    let casole = &trip.places[1];
    assert_eq!(casole.name, "Casole d'Elsa");
    assert_eq!(casole.file_ids, [ids("casole", 6), ids("near", 1)].concat());
    assert_eq!(casole.end, "2025-05-02 14:00:00");
    assert_eq!(trip.file_count, 17);
}
