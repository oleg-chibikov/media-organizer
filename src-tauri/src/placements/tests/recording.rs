use std::path::Path;

use crate::{
    models::SuggestionKind,
    placements::{forget_files, record_moves, MovedIntoPlace},
    test_support::placements::{moved_into, placement},
};

const ROOT: &str = "/Root";

fn usa(place: Option<&str>) -> crate::models::Placement {
    placement(SuggestionKind::Trip, 2025, "United States", place)
}

#[test]
fn later_batches_merge_into_the_same_record() {
    let root = Path::new(ROOT);
    let mut records = Vec::new();
    let nyc = usa(Some("New York"));

    record_moves(
        &mut records,
        root,
        &[moved_into(
            root,
            &nyc,
            "2025/United States/New York/10-29/a.jpg",
            "2025-10-29 10:00:00",
        )],
    );
    record_moves(
        &mut records,
        root,
        &[
            moved_into(
                root,
                &nyc,
                "2025/United States/New York/10-31/b.jpg",
                "2025-10-31 09:00:00",
            ),
            moved_into(
                root,
                &usa(None),
                "2025/United States/10-30/c.jpg",
                "2025-10-30 12:00:00",
            ),
        ],
    );

    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(
        (record.start.as_str(), record.end.as_str()),
        ("2025-10-29 10:00:00", "2025-10-31 09:00:00")
    );
    assert_eq!(record.places[0].files.len(), 2);
    assert_eq!(record.places[0].end, "2025-10-31 09:00:00");
    assert_eq!(
        record.root_files[0].relative_path,
        "2025/United States/10-30/c.jpg"
    );
}

#[test]
fn a_file_moved_to_a_new_day_folder_replaces_its_entry() {
    let root = Path::new(ROOT);
    let mut records = Vec::new();
    let vancouver = usa(Some("Vancouver"));
    let old = moved_into(
        root,
        &vancouver,
        "2025/United States/Vancouver/a.jpg",
        "2025-10-31 10:00:00",
    );
    record_moves(&mut records, root, std::slice::from_ref(&old));

    let relayout = MovedIntoPlace {
        source: old.destination.clone(),
        ..moved_into(
            root,
            &vancouver,
            "2025/United States/Vancouver/10-31/a.jpg",
            "2025-10-31 10:00:00",
        )
    };
    record_moves(&mut records, root, &[relayout]);

    let paths: Vec<&str> = records[0].places[0]
        .files
        .iter()
        .map(|file| file.relative_path.as_str())
        .collect();
    assert_eq!(paths, ["2025/United States/Vancouver/10-31/a.jpg"]);
}

#[test]
fn forgetting_every_file_drops_the_record() {
    let root = Path::new(ROOT);
    let mut records = Vec::new();
    let file = moved_into(
        root,
        &usa(Some("Boston")),
        "2025/United States/Boston/a.jpg",
        "2025-10-01 10:00:00",
    );
    record_moves(&mut records, root, std::slice::from_ref(&file));

    forget_files(
        &mut records,
        Path::new("/Other"),
        &[file.destination.as_path()],
    );
    assert_eq!(records.len(), 1);
    forget_files(&mut records, root, &[file.destination.as_path()]);

    assert!(records.is_empty());
}
