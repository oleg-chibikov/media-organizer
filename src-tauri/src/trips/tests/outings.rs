use super::{ids, Media, FRECHEN, ROYAL_BOTANIC_GARDEN, SYDNEY};
use crate::models::{DayLayout, SuggestionKind};

#[test]
fn a_botanic_garden_near_home_is_an_outing() {
    let mut media = Media::default();
    media
        .burst(
            "garden",
            "2025-03-01 10:00:00",
            6,
            10,
            Some(ROYAL_BOTANIC_GARDEN),
        )
        .burst("other", "2025-03-01 18:00:00", 1, 0, None);

    let suggestions = media.detect(Some(SYDNEY));

    assert_eq!(suggestions.len(), 1, "{suggestions:#?}");
    let outing = &suggestions[0];
    assert_eq!(outing.kind, SuggestionKind::Outing);
    assert_eq!(outing.title, "Royal Botanic Gardens");
    assert_eq!(outing.year, 2025);
    assert_eq!(outing.places.len(), 1);
    assert_eq!(outing.places[0].place_kind, "garden");
    assert_eq!(outing.places[0].day_layout, DayLayout::Single);
    assert_eq!(outing.places[0].file_ids, ids("garden", 6));
    assert!(outing.root_file_ids.is_empty());
}

#[test]
fn outings_form_with_an_unknown_home() {
    let mut media = Media::default();
    media.burst(
        "garden",
        "2025-03-01 10:00:00",
        5,
        10,
        Some(ROYAL_BOTANIC_GARDEN),
    );

    assert_eq!(media.detect(None).len(), 1);
}

#[test]
fn visits_in_one_year_merge_and_split_by_year() {
    let mut media = Media::default();
    media
        .burst(
            "march",
            "2025-03-01 10:00:00",
            3,
            10,
            Some(ROYAL_BOTANIC_GARDEN),
        )
        .burst(
            "june",
            "2025-06-01 10:00:00",
            3,
            10,
            Some(ROYAL_BOTANIC_GARDEN),
        )
        .burst(
            "next",
            "2026-01-01 10:00:00",
            5,
            10,
            Some(ROYAL_BOTANIC_GARDEN),
        );

    let suggestions = media.detect(Some(SYDNEY));

    assert_eq!(suggestions.len(), 2);
    let first = &suggestions[0].places[0];
    assert_eq!(first.file_ids, [ids("march", 3), ids("june", 3)].concat());
    assert_eq!(first.day_layout, DayLayout::MonthFolderDay);
    assert_eq!(suggestions[1].year, 2026);
}

#[test]
fn files_without_gps_join_an_outing_within_an_hour_on_the_same_day() {
    let mut media = Media::default();
    media
        .burst(
            "garden",
            "2025-03-01 10:00:00",
            5,
            10,
            Some(ROYAL_BOTANIC_GARDEN),
        )
        .burst("close", "2025-03-01 11:20:00", 1, 0, None)
        .burst("late", "2025-03-01 12:00:00", 1, 0, None);

    let outing = &media.detect(Some(SYDNEY))[0];

    assert_eq!(
        outing.places[0].file_ids,
        [ids("garden", 5), ids("close", 1)].concat()
    );
    assert_eq!(outing.file_count, 6);
}

#[test]
fn a_city_visit_is_no_outing() {
    let mut media = Media::default();
    media.burst("town", "2025-03-01 10:00:00", 8, 10, Some(FRECHEN));

    assert!(media.detect(None).is_empty());
}
