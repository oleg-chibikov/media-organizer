use crate::geo::{nearest_city, nearest_place, notable_place, Coordinates, Place, PlaceKind};

fn place_at(latitude: f64, longitude: f64) -> Place {
    nearest_place(Coordinates::new(latitude, longitude)).expect("a place near the point")
}

fn notable_at(latitude: f64, longitude: f64) -> Place {
    notable_place(Coordinates::new(latitude, longitude)).expect("a notable place")
}

#[test]
fn san_francisco_sights_are_san_francisco() {
    let sights = [
        (37.7793, -122.4193),
        (37.7544, -122.4477),
        (37.8080, -122.4177),
        (37.8267, -122.4230),
    ];
    for (latitude, longitude) in sights {
        assert_eq!(place_at(latitude, longitude).name, "San Francisco");
    }
}

#[test]
fn a_city_with_its_own_centre_keeps_its_name() {
    assert_eq!(place_at(37.8044, -122.2712).name, "Oakland");
}

#[test]
fn blue_mountains_towns_and_lookouts_are_the_national_park() {
    let spots = [
        (-33.7125, 150.3119),
        (-33.7320, 150.3120),
        (-33.7155, 150.3317),
        (-33.6280, 150.3110),
    ];
    for (latitude, longitude) in spots {
        let place = place_at(latitude, longitude);
        assert_eq!(place.name, "Blue Mountains National Park");
        assert_eq!(place.kind, PlaceKind::NationalPark);
    }
}

#[test]
fn royal_national_park_beaches_are_the_park() {
    for (latitude, longitude) in [(-34.1700, 151.0650), (-34.1370, 151.1170)] {
        assert_eq!(place_at(latitude, longitude).name, "Royal National Park");
        assert_eq!(notable_at(latitude, longitude).name, "Royal National Park");
    }
}

#[test]
fn harbour_suburbs_stay_sydney() {
    for (latitude, longitude) in [(-33.8915, 151.2767), (-33.7970, 151.1800)] {
        assert_eq!(place_at(latitude, longitude).name, "Sydney");
        let city = nearest_city(Coordinates::new(latitude, longitude)).expect("a city");
        assert_eq!(city.name, "Sydney");
    }
}

#[test]
fn a_garden_inside_sydney_is_still_an_outing() {
    let place = notable_at(-33.8642, 151.2166);
    assert!(place.name.contains("Botanic"), "{place:?}");
    assert_eq!(place.kind, PlaceKind::Garden);
    assert_eq!(place_at(-33.8642, 151.2166).name, "Sydney");
}

#[test]
fn kew_gardens_beat_the_palace_inside_them() {
    let place = notable_at(51.4787, -0.2956);
    assert!(place.name.contains("Kew"), "{place:?}");
    assert_eq!(place.kind, PlaceKind::Garden);
}
