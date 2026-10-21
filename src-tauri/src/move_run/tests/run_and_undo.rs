use std::fs;

use tempfile::tempdir;

use crate::{
    events,
    history::HistoryStore,
    models::{DateKind, FileKind, FileRecord, HistoryRunStatus},
    move_run::{start_move_run, MoveBatch},
    settings::SettingsStore,
    test_support::{
        events::RecordingSink,
        media::{file_record, ready_metadata, ready_plan},
    },
    undo_redo::{prepare_replay, ReplayDirection},
};

#[test]
fn moves_a_batch_records_history_and_undoes_it() {
    let temp = tempdir().expect("temp dir");
    let app_data = temp.path().join("app-data");
    let selected = temp.path().join("todo");
    let source = selected.join("nested").join("IMG_1.jpg");
    let destination = selected.join("Processed").join("2026").join("IMG_1.jpg");
    fs::create_dir_all(source.parent().expect("source parent")).expect("create source dir");
    fs::write(&source, b"image-bytes").expect("write source");
    let batch = MoveBatch {
        files: vec![FileRecord {
            source_path: source.to_string_lossy().to_string(),
            ..file_record("photo-1", "jpg", FileKind::Photo)
        }],
        metadata_results: vec![ready_metadata("photo-1", DateKind::TimezoneAware)],
        plans: vec![ready_plan("photo-1", &destination.to_string_lossy())],
    };
    let sink = RecordingSink::default();
    let settings = SettingsStore::in_app_data(&app_data);
    let history = HistoryStore::in_app_data(&app_data);

    start_move_run(&sink, &settings, &history, &selected, batch)
        .expect("move run should start")
        .execute(&sink);

    assert!(destination.exists() && !source.exists());
    assert_eq!(sink.payloads(events::MOVE_FINISHED)[0]["moved"], 1);
    let runs = history.list_runs().expect("list runs");
    assert_eq!(runs[0].run_status, HistoryRunStatus::Completed);
    assert!(runs[0].undo_available);
    let details = history.require_run(&runs[0].run_id).expect("run details");
    let image_size = Some(b"image-bytes".len() as u64);
    assert_eq!(details.operations[0].expected_source_size, image_size);
    assert_eq!(details.operations[0].expected_destination_size, image_size);

    prepare_replay(&history, &runs[0].run_id, ReplayDirection::Undo)
        .expect("run should be undoable")
        .run(&sink);

    assert!(source.exists() && !destination.exists());
    assert_eq!(sink.payloads(events::HISTORY_UNDO_PROGRESS).len(), 1);
    let runs = history.list_runs().expect("list runs");
    assert_eq!(runs[0].run_status, HistoryRunStatus::Undone);
    assert!(runs[0].redo_available);
}

#[test]
fn runs_one_replay_of_a_run_at_a_time() {
    let temp = tempdir().expect("temp dir");
    let app_data = temp.path().join("app-data");
    let selected = temp.path().join("todo");
    let source = selected.join("IMG_1.jpg");
    let destination = selected.join("Processed").join("IMG_1.jpg");
    fs::create_dir_all(&selected).expect("create selected folder");
    fs::write(&source, b"image-bytes").expect("write source");
    let batch = MoveBatch {
        files: vec![FileRecord {
            source_path: source.to_string_lossy().to_string(),
            ..file_record("photo-1", "jpg", FileKind::Photo)
        }],
        metadata_results: vec![ready_metadata("photo-1", DateKind::TimezoneAware)],
        plans: vec![ready_plan("photo-1", &destination.to_string_lossy())],
    };
    let (sink, history) = (
        RecordingSink::default(),
        HistoryStore::in_app_data(&app_data),
    );
    start_move_run(
        &sink,
        &SettingsStore::in_app_data(&app_data),
        &history,
        &selected,
        batch,
    )
    .expect("move run should start")
    .execute(&sink);
    let run_id = history.list_runs().expect("list runs")[0].run_id.clone();

    let undo = prepare_replay(&history, &run_id, ReplayDirection::Undo).expect("first undo");
    assert!(prepare_replay(&history, &run_id, ReplayDirection::Undo).is_err());
    assert!(history.delete_run(&run_id).is_err());
    assert!(destination.exists() && !source.exists());

    undo.run(&sink);
    assert!(source.exists() && !destination.exists());
    assert!(prepare_replay(&history, &run_id, ReplayDirection::Redo).is_ok());
}

#[test]
fn refuses_to_start_when_selected_folder_is_missing() {
    let temp = tempdir().expect("temp dir");
    let sink = RecordingSink::default();
    let batch = MoveBatch {
        files: Vec::new(),
        metadata_results: Vec::new(),
        plans: Vec::new(),
    };

    let started = start_move_run(
        &sink,
        &SettingsStore::in_app_data(temp.path()),
        &HistoryStore::in_app_data(temp.path()),
        &temp.path().join("missing"),
        batch,
    );

    assert!(started.is_err());
    assert!(sink.names().is_empty());
}
