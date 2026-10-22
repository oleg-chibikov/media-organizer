use super::placed_scan::{october_scan, trip, Scan};
use crate::{
    models::SuggestionKind,
    test_support::placements::{detected, place_group},
};

#[test]
fn a_trip_two_days_later_continues_the_placed_title_year_and_places() {
    let scan = october_scan();

    let found = scan.continue_all(vec![trip(
        "United States & Canada",
        "2025-11-01 09:00:00",
        "2025-11-02 09:00:00",
    )]);

    let continued = &found[0];
    assert_eq!(continued.title, "United States");
    assert_eq!(continued.year, 2025);
    let placed = continued.continues.as_ref().expect("continues");
    assert_eq!(
        (placed.start.as_str(), placed.end.as_str()),
        ("2025-10-28 10:00:00", "2025-10-31 18:00:00")
    );
    let vancouver = &continued.places[0];
    assert_eq!(vancouver.name, "Vancouver");
    assert_eq!(vancouver.start, "2025-11-01 09:00:00");
    let placed_names: Vec<(&str, &str)> = placed
        .places
        .iter()
        .map(|place| (place.name.as_str(), place.start.as_str()))
        .collect();
    assert_eq!(
        placed_names,
        [
            ("New York", "2025-10-28 10:00:00"),
            ("Vancouver", "2025-10-31 18:00:00")
        ]
    );
    assert_eq!(continued.places[1].name, "Toronto");
}

#[test]
fn a_january_part_of_a_new_year_trip_keeps_the_placed_year() {
    let scan = Scan::new();
    scan.place(
        &scan.root(),
        SuggestionKind::Trip,
        2025,
        "Japan",
        &[
            (
                "Tokyo",
                "2025/Japan/Tokyo/12-30/a.jpg",
                "2025-12-30 10:00:00",
            ),
            (
                "Tokyo",
                "2025/Japan/Tokyo/12-31/b.jpg",
                "2025-12-31 10:00:00",
            ),
        ],
    );

    let found = scan.continue_all(vec![trip(
        "Japan",
        "2026-01-01 09:00:00",
        "2026-01-02 09:00:00",
    )]);

    assert_eq!((found[0].year, found[0].title.as_str()), (2025, "Japan"));
}

#[test]
fn an_outing_to_the_same_place_continues_it() {
    let scan = Scan::new();
    scan.place(
        &scan.root(),
        SuggestionKind::Outing,
        2025,
        "Royal Botanic Garden",
        &[(
            "Royal Botanic Garden",
            "2025/Royal Botanic Garden/a.jpg",
            "2025-11-10 10:00:00",
        )],
    );
    let again = vec![place_group(
        "royal botanic garden",
        "2025-11-12 10:00:00",
        "2025-11-12 12:00:00",
        &["g"],
    )];

    let found = scan.continue_all(vec![detected(
        SuggestionKind::Outing,
        "royal botanic garden",
        again,
    )]);

    assert_eq!(found[0].title, "Royal Botanic Garden");
    assert_eq!(found[0].places[0].name, "Royal Botanic Garden");
    assert!(found[0].continues.is_some());
}
