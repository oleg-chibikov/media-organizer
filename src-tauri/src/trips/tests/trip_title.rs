use crate::trips::trip_title::trip_title;

#[test]
fn one_country_names_the_trip() {
    assert_eq!(trip_title(["Japan", "Japan"].into_iter()), "Japan");
}

#[test]
fn two_countries_are_joined_by_file_count() {
    let files = ["Italy", "France", "France", "Italy", "France"];
    assert_eq!(trip_title(files.into_iter()), "France & Italy");
}

#[test]
fn three_countries_name_the_top_one_and_more() {
    let files = ["Spain", "Portugal", "Portugal", "Morocco"];
    assert_eq!(trip_title(files.into_iter()), "Portugal & more");
}

#[test]
fn a_trip_with_no_known_country_is_a_trip() {
    assert_eq!(trip_title(std::iter::empty()), "Trip");
}
