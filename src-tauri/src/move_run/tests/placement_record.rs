use std::{fs, path::Path};

use tempfile::tempdir;

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
    undo_redo::{prepare_replay, ReplayDirection},
};

fn photo(id: &str, source: &Path) -> FileRecord {
    FileRecord {
        source_path: source.to_string_lossy().to_string(),
        ..file_record(id, "jpg", FileKind::Photo)
    }
}

fn placed_plan(id: &str, destination: &Path) -> MovePlan {
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
fn a_run_records_only_moved_files_and_undo_forgets_them() {
    let temp = tempdir().expect("temp dir");
    let (app_data, selected) = (temp.path().join("app-data"), temp.path().join("todo"));
    let tokyo = selected
        .join("Processed")
        .join("2025")
        .join("Japan")
        .join("Tokyo");
    let (moved, missing) = (selected.join("a.jpg"), selected.join("gone.jpg"));
    fs::create_dir_all(&selected).expect("selected folder");
    fs::write(&moved, b"a").expect("write source");
    let batch = MoveBatch {
        files: vec![photo("a", &moved), photo("gone", &missing)],
        metadata_results: vec![
            dated_metadata("a", "2025-04-01 10:00:00"),
            dated_metadata("gone", "2025-04-01 11:00:00"),
        ],
        plans: vec![
            placed_plan("a", &tokyo.join("a.jpg")),
            placed_plan("gone", &tokyo.join("gone.jpg")),
        ],
    };
    let sink = RecordingSink::default();
    let (history, placements) = (
        HistoryStore::in_app_data(&app_data),
        PlacementStore::in_app_data(&app_data),
    );
    let settings = SettingsStore::in_app_data(&app_data);

    start_move_run(&sink, &settings, &history, &placements, &selected, batch)
        .expect("run starts")
        .execute(&sink);

    let records = placements.load().expect("registry");
    assert_eq!(records.len(), 1);
    assert_eq!(
        (records[0].title.as_str(), records[0].year),
        ("Japan", 2025)
    );
    let files: Vec<&str> = records[0].places[0]
        .files
        .iter()
        .map(|file| file.relative_path.as_str())
        .collect();
    assert_eq!(files, ["2025/Japan/Tokyo/a.jpg"]);

    let run_id = history.list_runs().expect("runs")[0].run_id.clone();
    prepare_replay(&history, &placements, &run_id, ReplayDirection::Undo)
        .expect("undoable")
        .run(&sink);

    assert!(moved.exists());
    assert!(placements.load().expect("registry").is_empty());
}
