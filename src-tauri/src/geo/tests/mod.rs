mod areas;

use std::time::Instant;

use crate::geo::{
    country_of, distance_km, nearest_city, nearest_place, search_cities, Coordinates, Place,
    PlaceKind,
};

fn place_at(latitude: f64, longitude: f64) -> Place {
    nearest_place(Coordinates::new(latitude, longitude)).expect("a place near the point")
}

#[test]
fn yosemite_valley_is_in_a_yosemite_place() {
    let place = place_at(37.7456, -119.5936);
    assert!(place.name.contains("Yosemite"), "{place:?}");
    assert_ne!(place.kind, PlaceKind::City);
    assert_eq!(place.country_name, "United States");
}

#[test]
fn moscow_center_is_in_moscow_russia() {
    let point = Coordinates::new(55.7558, 37.6173);
    let city = nearest_city(point).expect("a city");
    assert_eq!(city.name, "Moscow");
    assert_eq!(city.country_name, "Russia");
    let country = country_of(point).expect("a country");
    assert_eq!(country, ("RU".to_owned(), "Russia".to_owned()));
}

#[test]
fn mid_pacific_has_no_place() {
    assert_eq!(nearest_place(Coordinates::new(-30.0, -140.0)), None);
}

#[test]
fn points_near_the_poles_and_date_line_do_not_panic() {
    for (latitude, longitude) in [(90.0, 0.0), (-90.0, 180.0), (64.0, -179.99), (64.0, 179.99)] {
        let _ = nearest_place(Coordinates::new(latitude, longitude));
    }
}

#[test]
fn paris_to_london_is_about_344_km() {
    let paris = Coordinates::new(48.8566, 2.3522);
    let london = Coordinates::new(51.5074, -0.1278);
    assert!((distance_km(paris, london) - 344.0).abs() < 2.0);
}

#[test]
fn lookups_take_microseconds_once_loaded() {
    let center = Coordinates::new(48.8566, 2.3522);
    nearest_place(center);
    let started = Instant::now();
    for step in 0..1000 {
        nearest_place(Coordinates::new(48.0 + f64::from(step) * 0.001, 2.0));
    }
    let per_lookup = started.elapsed() / 1000;
    assert!(per_lookup.as_micros() < 1000, "{per_lookup:?} per lookup");
}

#[test]
fn city_search_ignores_case_and_accents() {
    for query in ["zur", "ZÜR", " Zürich ", "zuerich"] {
        let first = search_cities(query, 10).into_iter().next().expect("a city");
        assert_eq!(first.name, "Zuerich", "{query}");
        assert_eq!(first.country_name, "Switzerland");
    }
}

#[test]
fn city_search_puts_the_biggest_prefix_match_first() {
    let found = search_cities("sydney", 10);

    let first_two: Vec<&str> = found[..2]
        .iter()
        .map(|place| place.country_name.as_str())
        .collect();
    assert_eq!(first_two, ["Australia", "Canada"], "{found:?}");
    assert!(found.iter().all(|place| place.kind == PlaceKind::City));
}

#[test]
fn city_search_lists_containing_names_after_prefixes() {
    let names: Vec<String> = search_cities("sydney", 10)
        .into_iter()
        .map(|place| place.name)
        .collect();

    let expected = [
        "Sydney",
        "Sydney",
        "Sydney Mines",
        "Sydney Forks",
        "North Sydney",
        "Hampden Sydney",
    ];
    assert_eq!(names, expected);
}

#[test]
fn city_search_returns_at_most_the_limit() {
    assert_eq!(search_cities("san", 10).len(), 10);
    assert!(search_cities("  ", 10).is_empty());
}
