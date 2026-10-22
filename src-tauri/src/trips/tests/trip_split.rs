use super::{ids, Media, FRECHEN, SYDNEY};
use crate::models::SuggestionKind;

#[test]
fn a_gap_over_three_days_starts_a_new_trip() {
    let mut media = Media::default();
    media
        .burst("first", "2025-06-01 10:00:00", 5, 60, Some(FRECHEN))
        .burst("second", "2025-06-04 09:00:00", 5, 60, Some(FRECHEN))
        .burst("third", "2025-06-10 10:00:00", 5, 60, Some(FRECHEN));

    let trips = media.detect(Some(SYDNEY));

    assert_eq!(trips.len(), 2, "{trips:#?}");
    assert!(trips.iter().all(|trip| trip.kind == SuggestionKind::Trip));
    assert_eq!(trips[0].file_count, 10);
    assert_eq!(trips[0].start, "2025-06-01 10:00:00");
    assert_eq!(trips[0].end, "2025-06-04 13:00:00");
    assert_eq!(trips[1].places[0].file_ids, ids("third", 5));
}

#[test]
fn a_run_under_five_files_is_no_trip() {
    let mut media = Media::default();
    media.burst("short", "2025-06-01 10:00:00", 4, 60, Some(FRECHEN));

    assert!(media.detect(Some(SYDNEY)).is_empty());
}

#[test]
fn an_unknown_home_gives_no_trips() {
    let mut media = Media::default();
    media.burst("away", "2025-06-01 10:00:00", 8, 60, Some(FRECHEN));

    assert!(media.detect(None).is_empty());
}

#[test]
fn a_trip_abroad_splits_into_one_suggestion_per_country() {
    let mut media = Media::default();
    media
        .burst("de", "2024-12-31 20:00:00", 6, 60, Some(FRECHEN))
        .burst("fr", "2025-01-01 03:00:00", 5, 60, Some(super::RILLIEUX));

    let trips = media.detect(Some(SYDNEY));

    let titles: Vec<_> = trips
        .iter()
        .map(|trip| (trip.title.as_str(), trip.year))
        .collect();
    assert_eq!(titles, [("Germany", 2024), ("France", 2025)]);
    assert_eq!(trips[0].country.as_deref(), Some("Germany"));
    assert_eq!(trips[1].country.as_deref(), Some("France"));
    assert_ne!(trips[0].id, trips[1].id);
    assert!(trips[0].id.starts_with("trip-"), "{}", trips[0].id);
}
