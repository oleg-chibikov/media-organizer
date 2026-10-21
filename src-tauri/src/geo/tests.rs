use std::time::Instant;

use crate::geo::{
    country_of, distance_km, nearest_city, nearest_place, Coordinates, Place, PlaceKind,
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
fn royal_botanic_garden_sydney_is_found() {
    let place = place_at(-33.8642, 151.2166);
    assert!(place.name.contains("Botanic"), "{place:?}");
    assert_eq!(place.kind, PlaceKind::Garden);
    assert_eq!(place.country_name, "Australia");
}

#[test]
fn kew_gardens_beat_the_palace_inside_them() {
    let place = place_at(51.4787, -0.2956);
    assert!(place.name.contains("Kew"), "{place:?}");
    assert_eq!(place.kind, PlaceKind::Garden);
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
