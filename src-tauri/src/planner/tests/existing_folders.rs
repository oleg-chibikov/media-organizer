use std::{collections::HashMap, fs, path::Path};

use tempfile::tempdir;

use super::{dated, photo_record, place};
use crate::{
    models::FilePlace,
    planner::{plan_files, PlaceFolders, Reservations},
    test_support::media::dated_metadata,
};

/// Plans one-day files into `root` after reusing its folders, and returns each folder
/// relative to `root`.
fn plan_into(root: &Path, files: &[(&str, &str, FilePlace)]) -> Vec<String> {
    let records: Vec<_> = files
        .iter()
        .map(|(id, ..)| photo_record(id, "jpg"))
        .collect();
    let metadata: Vec<_> = files
        .iter()
        .map(|(id, date, _)| dated_metadata(id, date))
        .collect();
    let places: HashMap<String, FilePlace> = files
        .iter()
        .map(|(id, _, place)| ((*id).to_owned(), place.clone()))
        .collect();
    let mut folders = PlaceFolders::new(&places, &metadata).expect("valid places");
    folders.reuse_existing(root);
    plan_files(
        &dated(root),
        &records,
        &metadata,
        &folders,
        &mut Reservations::default(),
    )
    .plans
    .iter()
    .map(|plan| {
        let dir = Path::new(&plan.destination_dir)
            .strip_prefix(root)
            .expect("under root");
        dir.to_string_lossy().replace('\\', "/")
    })
    .collect()
}

fn osaka_day() -> Vec<(&'static str, &'static str, FilePlace)> {
    vec![("a", "2024-03-15 10:00:00", place("Osaka", None))]
}

#[test]
fn one_day_joins_the_place_folder_of_its_month() {
    let root = tempdir().expect("temp dir");
    fs::create_dir_all(root.path().join("2024/3/Osaka/14")).expect("folder");

    assert_eq!(plan_into(root.path(), &osaka_day()), ["2024/3/Osaka/15"]);
}

#[test]
fn one_day_joins_the_place_folder_of_its_year() {
    let root = tempdir().expect("temp dir");
    fs::create_dir_all(root.path().join("2024/Osaka/1/2")).expect("folder");

    assert_eq!(plan_into(root.path(), &osaka_day()), ["2024/Osaka/3/15"]);
}

#[test]
fn a_place_joins_the_existing_country_and_then_its_own_folder() {
    let root = tempdir().expect("temp dir");
    fs::create_dir_all(root.path().join("2024/Japan/Kyoto")).expect("folder");
    let kyoto = [("a", "2024-03-15 10:00:00", place("Kyoto", Some("Japan")))];
    let osaka = [("b", "2024-03-15 10:00:00", place("Osaka", Some("Japan")))];

    assert_eq!(plan_into(root.path(), &kyoto), ["2024/Japan/Kyoto/3/15"]);
    assert_eq!(plan_into(root.path(), &osaka), ["2024/Japan/3/15/Osaka"]);
}

#[test]
fn a_deeper_folder_of_the_same_place_is_left_alone() {
    let root = tempdir().expect("temp dir");
    fs::create_dir_all(root.path().join("2024/3/1/Osaka")).expect("folder");
    let month = [
        ("a", "2024-03-15 10:00:00", place("Osaka", None)),
        ("b", "2024-03-16 10:00:00", place("Osaka", None)),
    ];

    assert_eq!(
        plan_into(root.path(), &month),
        ["2024/3/Osaka/15", "2024/3/Osaka/16"]
    );
}
