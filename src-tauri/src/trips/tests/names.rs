use crate::{
    geo::{Place, PlaceKind},
    trips::{place_names::place_folder_names, place_table::PlaceTable},
};

fn park(name: &str, admin1: Option<&str>, latitude: f64) -> Place {
    Place {
        name: name.to_owned(),
        kind: PlaceKind::Park,
        admin1: admin1.map(str::to_owned),
        country_code: "US".to_owned(),
        country_name: "United States".to_owned(),
        latitude,
        longitude: -74.0,
        population: 0,
    }
}

#[test]
fn places_sharing_a_name_get_their_region() {
    let mut places = PlaceTable::default();
    let new_york = places.intern(park("Central Park", Some("New York"), 40.78));
    let colorado = places.intern(park("Central Park", Some("Colorado"), 39.7));
    let other = places.intern(park("Bryant Park", Some("New York"), 40.75));

    let names = place_folder_names(&[new_york, colorado, other], &places);

    assert_eq!(
        names,
        [
            "Central Park (New York)",
            "Central Park (Colorado)",
            "Bryant Park"
        ]
    );
}

#[test]
fn a_shared_name_with_no_region_gets_the_country() {
    let mut places = PlaceTable::default();
    let first = places.intern(park("Echo Lake", None, 41.0));
    let second = places.intern(park("Echo Lake", Some("Maine"), 44.0));

    let names = place_folder_names(&[first, second], &places);

    assert_eq!(names, ["Echo Lake (United States)", "Echo Lake (Maine)"]);
}

#[test]
fn names_are_safe_folder_names() {
    let mut places = PlaceTable::default();
    let place = places.intern(park("Fort Worth: Water Gardens / Plaza.", None, 32.7));

    let names = place_folder_names(&[place], &places);

    assert_eq!(names, ["Fort Worth_ Water Gardens _ Plaza"]);
}
