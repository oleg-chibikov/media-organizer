use std::path::{Path, PathBuf};

use super::{
    dated, photo_record,
    place_fixtures::{group, suggestion},
};
use crate::{
    events,
    models::{PlaceSuggestion, SuggestionKind},
    planner::{stream_move_plan, PlaceFolders, Reservations},
    test_support::{events::RecordingSink, media::dated_metadata},
};

/// Plans `(file id, chosen date)` pairs and returns each destination folder.
fn planned_dirs(suggestions: &[PlaceSuggestion], files: &[(&str, &str)]) -> Vec<PathBuf> {
    let records: Vec<_> = files
        .iter()
        .map(|(id, _)| photo_record(id, "jpg"))
        .collect();
    let metadata: Vec<_> = files
        .iter()
        .map(|(id, date)| dated_metadata(id, date))
        .collect();
    let layout = dated(Path::new("Root"));
    let folders = PlaceFolders::new(suggestions, &metadata).expect("valid suggestions");
    let sink = RecordingSink::default();
    stream_move_plan(
        &sink,
        0,
        &layout,
        &records,
        &metadata,
        &folders,
        &mut Reservations::default(),
    );
    sink.payloads(events::PLAN_FILE)
        .iter()
        .map(|payload| PathBuf::from(payload["plan"]["destination_dir"].as_str().expect("a dir")))
        .collect()
}

fn root(parts: &[&str]) -> PathBuf {
    parts
        .iter()
        .fold(PathBuf::from("Root"), |dir, part| dir.join(part))
}

#[test]
fn accepted_trip_files_go_to_place_and_day_folders() {
    let rome = group(
        "Rome",
        "2025-10-28 10:00:00",
        "2025-11-02 18:00:00",
        &["a", "b"],
    );
    let mut trip = suggestion(SuggestionKind::Trip, "Italy & France", vec![rome]);
    trip.root_file_ids = vec!["c".to_owned()];

    let dirs = planned_dirs(
        &[trip],
        &[
            ("a", "2025-10-28 10:00:00"),
            ("b", "2025-11-02 18:00:00"),
            ("c", "2025-10-30 12:00:00"),
            ("other", "2025-10-30 12:00:00"),
        ],
    );

    assert_eq!(
        dirs,
        [
            root(&["2025", "Italy & France", "Rome", "10-28"]),
            root(&["2025", "Italy & France", "Rome", "11-02"]),
            root(&["2025", "Italy & France", "10-30"]),
            root(&["2025", "10", "30"]),
        ]
    );
}

#[test]
fn outings_use_one_folder_for_one_day_and_month_folders_past_a_month() {
    let garden = group(
        "Royal Botanic Gardens",
        "2025-03-01 10:00:00",
        "2025-03-01 11:00:00",
        &["a"],
    );
    let zoo = group(
        "Taronga Zoo",
        "2025-03-01 10:00:00",
        "2025-04-20 10:00:00",
        &["b"],
    );
    let suggestions = [
        suggestion(
            SuggestionKind::Outing,
            "Royal Botanic Gardens",
            vec![garden],
        ),
        suggestion(SuggestionKind::Outing, "Taronga Zoo", vec![zoo]),
    ];

    let dirs = planned_dirs(
        &suggestions,
        &[("a", "2025-03-01 10:00:00"), ("b", "2025-04-20 10:00:00")],
    );

    assert_eq!(
        dirs,
        [
            root(&["2025", "Royal Botanic Gardens"]),
            root(&["2025", "Taronga Zoo", "4", "20"]),
        ]
    );
}

#[test]
fn a_file_date_outside_the_suggested_span_widens_the_day_layout() {
    let garden = group(
        "Garden",
        "2025-03-01 10:00:00",
        "2025-03-01 11:00:00",
        &["a"],
    );
    let outing = suggestion(SuggestionKind::Outing, "Garden", vec![garden]);

    let dirs = planned_dirs(&[outing], &[("a", "2025-03-03 10:00:00")]);

    assert_eq!(dirs, [root(&["2025", "Garden", "03-03"])]);
}

#[test]
fn no_suggestions_keep_the_dated_folders() {
    let dirs = planned_dirs(&[], &[("a", "2025-03-01 10:00:00")]);

    assert_eq!(dirs, [root(&["2025", "3", "1"])]);
}
