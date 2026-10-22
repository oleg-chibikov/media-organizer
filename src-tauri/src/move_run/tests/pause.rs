use std::fs;

use tempfile::tempdir;

use crate::{
    events,
    history::HistoryStore,
    models::{DateKind, FileKind, FileRecord},
    move_run::{start_move_run, MoveBatch},
    settings::SettingsStore,
    test_support::{
        events::RecordingSink,
        media::{file_record, ready_metadata, ready_plan},
    },
};

#[test]
fn a_paused_run_leaves_unreached_files_at_the_source() {
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
    let sink = RecordingSink::default();
    let history = HistoryStore::in_app_data(&app_data);
    let token = tokio_util::sync::CancellationToken::new();
    token.cancel();

    let reached = start_move_run(
        &sink,
        &SettingsStore::in_app_data(&app_data),
        &history,
        &selected,
        batch,
    )
    .expect("move run should start")
    .execute(&sink, &token);

    assert!(reached.is_empty());
    assert!(source.exists() && !destination.exists());
    assert!(sink.payloads(events::MOVE_FILE).is_empty());
    let finished = &sink.payloads(events::MOVE_FINISHED)[0];
    assert_eq!(finished["results"].as_array().map(Vec::len), Some(0));
    let run_id = &history.list_runs().expect("list runs")[0].run_id;
    let details = history.require_run(run_id).expect("run details");
    assert!(details.operations.is_empty());
}
