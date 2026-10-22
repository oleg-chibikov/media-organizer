use std::{collections::HashMap, fs, path::Path};

use tempfile::tempdir;

use super::{dated, photo_record, place};
use crate::{
    models::FilePlace,
    planner::{plan_files, PlaceFolders, Reservations},
    test_support::media::dated_metadata,
};

/// The folder of each scan file, and each library move as `from -> to`, relative to `root`.
fn plan_with_library(
    root: &Path,
    selected: &Path,
    files: &[(&str, &str, FilePlace)],
) -> (Vec<String>, Vec<String>) {
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
    let mut reservations = Reservations::default();
    let nearby = folders.gather_nearby(root);
    folders.reuse_existing(root);
    let library = folders.library_moves(root, &nearby, selected, &mut reservations);
    let relative = |path: &str| {
        let path = Path::new(path).strip_prefix(root).expect("under root");
        path.to_string_lossy().replace('\\', "/")
    };
    let planned = plan_files(
        &dated(root),
        &records,
        &metadata,
        &folders,
        &mut reservations,
    );
    let dirs = planned
        .plans
        .iter()
        .map(|p| relative(&p.destination_dir))
        .collect();
    let moves = library
        .plans
        .iter()
        .map(|plan| {
            format!(
                "{} -> {}",
                relative(&plan.file_id),
                relative(&plan.destination_path)
            )
        })
        .collect();
    (dirs, moves)
}

fn photo_at(root: &Path, relative: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("parent")).expect("folder");
    fs::write(path, b"photo").expect("photo");
}

fn dubai_on(date: &'static str) -> Vec<(&'static str, &'static str, FilePlace)> {
    vec![("new", date, place("Dubai", None))]
}

#[test]
fn an_earlier_day_of_the_month_moves_into_the_month_folder() {
    let root = tempdir().expect("temp dir");
    photo_at(root.path(), "2024/3/4/Dubai/a.jpg");
    let (dirs, moves) = plan_with_library(
        root.path(),
        Path::new("/elsewhere"),
        &dubai_on("2024-03-07 10:00:00"),
    );

    assert_eq!(dirs, ["2024/3/Dubai/7"]);
    assert_eq!(moves, ["2024/3/4/Dubai/a.jpg -> 2024/3/Dubai/4/a.jpg"]);
}

#[test]
fn a_day_of_the_next_month_raises_the_place_to_its_year() {
    let root = tempdir().expect("temp dir");
    photo_at(root.path(), "2024/2/Dubai/27/a.jpg");
    let (dirs, moves) = plan_with_library(
        root.path(),
        Path::new("/elsewhere"),
        &dubai_on("2024-03-02 10:00:00"),
    );

    assert_eq!(dirs, ["2024/Dubai/3/2"]);
    assert_eq!(moves, ["2024/2/Dubai/27/a.jpg -> 2024/Dubai/2/27/a.jpg"]);
}

#[test]
fn a_day_two_months_away_stays_apart() {
    let root = tempdir().expect("temp dir");
    photo_at(root.path(), "2024/1/5/Dubai/a.jpg");
    let (dirs, moves) = plan_with_library(
        root.path(),
        Path::new("/elsewhere"),
        &dubai_on("2024-03-07 10:00:00"),
    );

    assert_eq!(dirs, ["2024/3/7/Dubai"]);
    assert!(moves.is_empty());
}

#[test]
fn the_country_folder_moves_up_with_its_place() {
    let root = tempdir().expect("temp dir");
    photo_at(root.path(), "2024/3/4/UAE/Dubai/a.jpg");
    let files = [("new", "2024-03-07 10:00:00", place("Dubai", Some("UAE")))];
    let (dirs, moves) = plan_with_library(root.path(), Path::new("/elsewhere"), &files);

    assert_eq!(dirs, ["2024/3/UAE/Dubai/7"]);
    assert_eq!(
        moves,
        ["2024/3/4/UAE/Dubai/a.jpg -> 2024/3/UAE/Dubai/4/a.jpg"]
    );
}

#[test]
fn files_in_the_scanned_folder_and_other_files_stay_out_of_the_library_moves() {
    let root = tempdir().expect("temp dir");
    photo_at(root.path(), "2024/3/4/Dubai/a.jpg");
    photo_at(root.path(), "2024/3/4/Dubai/notes.txt");
    photo_at(root.path(), "2024/3/5/Dubai/b.jpg");
    let selected = root.path().join("2024/3/5");
    let (_, moves) = plan_with_library(root.path(), &selected, &dubai_on("2024-03-07 10:00:00"));

    assert_eq!(moves, ["2024/3/4/Dubai/a.jpg -> 2024/3/Dubai/4/a.jpg"]);
}
