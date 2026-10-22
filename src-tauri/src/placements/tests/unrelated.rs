use super::placed_scan::{october_scan, trip};
use crate::{
    models::SuggestionKind,
    test_support::placements::{detected, place_group},
};

#[test]
fn unrelated_trips_and_outings_start_new_folders() {
    let scan = october_scan();
    let elsewhere = scan.temp.path().join("Other root");
    scan.place(
        &elsewhere,
        SuggestionKind::Trip,
        2025,
        "Canada",
        &[(
            "Toronto",
            "2025/Canada/Toronto/a.jpg",
            "2025-11-20 10:00:00",
        )],
    );
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
    let other_outing = vec![place_group(
        "Taronga Zoo",
        "2025-11-11 10:00:00",
        "2025-11-11 12:00:00",
        &["z"],
    )];

    let found = scan.continue_all(vec![
        trip("Canada", "2025-11-05 09:00:00", "2025-11-06 09:00:00"),
        trip("Canada", "2025-11-21 09:00:00", "2025-11-22 09:00:00"),
        detected(SuggestionKind::Outing, "Taronga Zoo", other_outing),
    ]);

    assert!(
        found
            .iter()
            .all(|suggestion| suggestion.continues.is_none()),
        "{found:?}"
    );
}

#[test]
fn a_placed_trip_whose_folder_was_deleted_is_ignored() {
    let scan = october_scan();
    std::fs::remove_dir_all(scan.root().join("2025").join("United States")).expect("delete");

    let found = scan.continue_all(vec![trip(
        "Canada",
        "2025-11-01 09:00:00",
        "2025-11-02 09:00:00",
    )]);

    assert!(found[0].continues.is_none());
    assert_eq!(found[0].title, "Canada");
}
