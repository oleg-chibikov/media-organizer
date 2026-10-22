use std::{fs, path::Path};

use tempfile::tempdir;
use tokio_util::sync::CancellationToken;

use crate::{
    history::HistoryStore,
    models::{FileKind, FileRecord, MovePlan, SuggestionKind},
    move_run::{start_move_run, MoveBatch},
    placements::PlacementStore,
    settings::SettingsStore,
    test_support::{
        events::RecordingSink,
        media::{dated_metadata, file_record, ready_plan},
        placements::placement,
    },
};

fn placed_file(id: &str, source: &Path) -> FileRecord {
    fs::create_dir_all(source.parent().expect("a parent")).expect("create day folder");
    fs::write(source, id.as_bytes()).expect("write placed file");
    FileRecord {
        source_path: source.to_string_lossy().to_string(),
        ..file_record(id, "jpg", FileKind::Photo)
    }
}

fn tokyo_plan(id: &str, destination: &Path) -> MovePlan {
    MovePlan {
        placement: Some(placement(
            SuggestionKind::Trip,
            2025,
            "Japan",
            Some("Tokyo"),
        )),
        ..ready_plan(id, &destination.to_string_lossy())
    }
}

#[test]
fn a_relayout_removes_emptied_day_folders_and_keeps_the_rest() {
    let temp = tempdir().expect("temp dir");
    let (app_data, selected) = (temp.path().join("app-data"), temp.path().join("todo"));
    let tokyo = selected
        .join("Processed")
        .join("2025")
        .join("Japan")
        .join("Tokyo");
    let kept = tokyo.join("10-29").join("kept.jpg");
    let batch = MoveBatch {
        files: vec![
            placed_file("a", &tokyo.join("10-28").join("a.jpg")),
            placed_file("b", &tokyo.join("10-29").join("b.jpg")),
        ],
        metadata_results: vec![
            dated_metadata("a", "2025-10-28 10:00:00"),
            dated_metadata("b", "2025-10-29 10:00:00"),
        ],
        plans: vec![
            tokyo_plan("a", &tokyo.join("10").join("28").join("a.jpg")),
            tokyo_plan("b", &tokyo.join("10").join("29").join("b.jpg")),
        ],
    };
    fs::write(&kept, b"kept").expect("write kept file");
    let sink = RecordingSink::default();

    start_move_run(
        &sink,
        &SettingsStore::in_app_data(&app_data),
        &HistoryStore::in_app_data(&app_data),
        &PlacementStore::in_app_data(&app_data),
        &selected,
        batch,
    )
    .expect("run starts")
    .execute(&sink, &CancellationToken::new());

    assert!(tokyo.join("10").join("28").join("a.jpg").exists());
    assert!(!tokyo.join("10-28").exists());
    assert!(kept.exists());
    assert!(tokyo.is_dir());
}

#[test]
fn a_file_moved_out_of_its_place_folder_keeps_that_folder() {
    let temp = tempdir().expect("temp dir");
    let (app_data, selected) = (temp.path().join("app-data"), temp.path().join("todo"));
    let tokyo = selected
        .join("Processed")
        .join("2025")
        .join("Japan")
        .join("Tokyo");
    let batch = MoveBatch {
        files: vec![placed_file("a", &tokyo.join("a.jpg"))],
        metadata_results: vec![dated_metadata("a", "2025-10-28 10:00:00")],
        plans: vec![tokyo_plan("a", &tokyo.join("10-28").join("a.jpg"))],
    };
    let sink = RecordingSink::default();

    start_move_run(
        &sink,
        &SettingsStore::in_app_data(&app_data),
        &HistoryStore::in_app_data(&app_data),
        &PlacementStore::in_app_data(&app_data),
        &selected,
        batch,
    )
    .expect("run starts")
    .execute(&sink, &CancellationToken::new());

    assert!(tokyo.join("10-28").join("a.jpg").exists());
    assert!(tokyo.is_dir());
}
