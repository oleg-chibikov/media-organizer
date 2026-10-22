use super::placed_scan::Scan;
use crate::{
    models::{PlaceSuggestion, Placement, SuggestionKind},
    test_support::placements::{detected, moved_into, place_group, place_on_disk, placement},
};

fn abroad(mut placement: Placement, country: &str) -> Placement {
    placement.country = Some(country.to_owned());
    placement
}

fn outing(country: Option<&str>) -> PlaceSuggestion {
    let date = "2025-03-02 10:00:00";
    let zoo = place_group("Zoo", date, date, &["z1"]);
    let mut suggestion = detected(SuggestionKind::Outing, "Zoo", vec![zoo]);
    suggestion.country = country.map(str::to_owned);
    suggestion
}

/// A Zoo outing placed under `2025/Australia/Zoo`, with home outside Australia.
fn placed_zoo_abroad() -> Scan {
    let scan = Scan::new();
    let zoo = abroad(
        placement(SuggestionKind::Outing, 2025, "Zoo", Some("Zoo")),
        "Australia",
    );
    let moved = moved_into(
        &scan.root(),
        &zoo,
        "2025/Australia/Zoo/a.jpg",
        "2025-03-01 10:00:00",
    );
    place_on_disk(&scan.store(), &scan.root(), &[moved]);
    scan
}

#[test]
fn an_outing_abroad_continues_the_record_in_the_same_country_folder() {
    let scan = placed_zoo_abroad();

    let found = scan.continue_all(vec![outing(Some("Australia"))]);

    assert!(found[0].continues.is_some());
}

#[test]
fn an_outing_at_home_ignores_a_record_in_a_country_folder() {
    let scan = placed_zoo_abroad();

    let found = scan.continue_all(vec![outing(None), outing(Some("New Zealand"))]);

    assert!(found
        .iter()
        .all(|suggestion| suggestion.continues.is_none()));
}
