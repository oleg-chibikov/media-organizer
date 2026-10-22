use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::{dated, place, planned_dirs};
use crate::{models::FilePlace, planner::PlaceFolders};

fn path(parts: &str) -> PathBuf {
    parts
        .split('/')
        .fold(PathBuf::from("Media"), |dir, part| dir.join(part))
}

fn plan(files: &[(&str, &str, Option<FilePlace>)]) -> Vec<PathBuf> {
    planned_dirs(&dated(Path::new("Media")), files)
}

#[test]
fn a_country_and_place_over_months_sit_in_the_year() {
    let kyoto = || Some(place("Kyoto", Some("Japan")));
    let dirs = plan(&[
        ("a", "2024-03-15 10:00:00", kyoto()),
        ("b", "2024-04-02 10:00:00", kyoto()),
    ]);

    assert_eq!(
        dirs,
        [path("2024/Japan/Kyoto/3/15"), path("2024/Japan/Kyoto/4/2")]
    );
}

#[test]
fn places_of_one_day_sit_in_that_day_inside_a_longer_country() {
    let dirs = plan(&[
        (
            "a",
            "2024-03-15 09:00:00",
            Some(place("Kyoto", Some("Japan"))),
        ),
        (
            "b",
            "2024-03-15 18:00:00",
            Some(place("Osaka", Some("Japan"))),
        ),
        (
            "c",
            "2024-04-20 10:00:00",
            Some(place("Tokyo", Some("Japan"))),
        ),
    ]);

    assert_eq!(
        dirs,
        [
            path("2024/Japan/3/15/Kyoto"),
            path("2024/Japan/3/15/Osaka"),
            path("2024/Japan/4/20/Tokyo"),
        ]
    );
}

#[test]
fn a_place_over_days_of_one_month_sits_in_the_month() {
    let dirs = plan(&[
        (
            "a",
            "2024-03-14 10:00:00",
            Some(place("Kyoto", Some("Japan"))),
        ),
        (
            "b",
            "2024-03-15 10:00:00",
            Some(place("Kyoto", Some("Japan"))),
        ),
        (
            "c",
            "2024-05-01 10:00:00",
            Some(place("Tokyo", Some("Japan"))),
        ),
    ]);

    assert_eq!(dirs[0], path("2024/Japan/3/Kyoto/14"));
    assert_eq!(dirs[1], path("2024/Japan/3/Kyoto/15"));
}

#[test]
fn a_place_in_home_country_has_no_country_and_files_with_no_place_keep_date_folders() {
    let dirs = plan(&[
        ("a", "2024-03-15 10:00:00", Some(place("Melbourne", None))),
        ("b", "2024-06-01 10:00:00", Some(place("Melbourne", None))),
        ("c", "2024-03-15 11:00:00", None),
    ]);

    assert_eq!(
        dirs,
        [
            path("2024/Melbourne/3/15"),
            path("2024/Melbourne/6/1"),
            path("2024/3/15")
        ]
    );
}

#[test]
fn years_dont_share_a_folder() {
    let dirs = plan(&[
        (
            "a",
            "2023-12-31 22:00:00",
            Some(place("Kyoto", Some("Japan"))),
        ),
        (
            "b",
            "2024-01-01 09:00:00",
            Some(place("Kyoto", Some("Japan"))),
        ),
    ]);

    assert_eq!(
        dirs,
        [path("2023/12/31/Japan/Kyoto"), path("2024/1/1/Japan/Kyoto")]
    );
}

#[test]
fn a_place_name_that_is_a_path_is_an_error() {
    let places = HashMap::from([("a".to_owned(), place("../Kyoto", None))]);
    let metadata = [crate::test_support::media::dated_metadata(
        "a",
        "2024-03-15 10:00:00",
    )];

    assert!(PlaceFolders::new(&places, &metadata).is_err());
}
