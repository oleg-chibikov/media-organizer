use std::{collections::HashMap, path::Path};

use super::{dated, photo_record, place};
use crate::{
    models::{FilePlace, HomeMatch},
    planner::{plan_files, PlaceFolders, Reservations},
    test_support::media::dated_metadata,
};

#[test]
fn file_names_carry_the_place_and_country_home_city_included() {
    let sydney = FilePlace {
        home: HomeMatch::City,
        ..place("Sydney", None)
    };
    let files = [
        ("a", Some(sydney)),
        ("b", Some(place("Melbourne", None))),
        ("c", Some(place("Kyoto", Some("Japan")))),
        ("d", None),
    ];
    let records: Vec<_> = files
        .iter()
        .map(|(id, _)| photo_record(id, "jpg"))
        .collect();
    let metadata: Vec<_> = files
        .iter()
        .map(|(id, _)| dated_metadata(id, "2026-03-05 07:20:17"))
        .collect();
    let places: HashMap<String, FilePlace> = files
        .iter()
        .filter_map(|(id, place)| Some(((*id).to_owned(), place.clone()?)))
        .collect();
    let folders = PlaceFolders::new(&places, &metadata).expect("valid places");

    let planned = plan_files(
        &dated(Path::new("Media")),
        &records,
        &metadata,
        &folders,
        &mut Reservations::default(),
    );

    let names: Vec<_> = planned
        .plans
        .iter()
        .map(|p| p.destination_filename.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "2026-03-05 07-20-17 Sydney, Australia (a) - iPhone 15 Pro.jpg",
            "2026-03-05 07-20-17 Melbourne, Australia (b) - iPhone 15 Pro.jpg",
            "2026-03-05 07-20-17 Kyoto, Japan (c) - iPhone 15 Pro.jpg",
            "2026-03-05 07-20-17 (d) - iPhone 15 Pro.jpg",
        ]
    );
}
