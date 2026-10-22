use std::{fs, path::Path};

use chrono::{DateTime, TimeZone, Utc};
use tempfile::tempdir;

use crate::{
    history::{
        index_updates::upsert_history_index_entry,
        paths::{
            checked_run_log_path, history_index_path, history_root_from_app_data, history_runs_dir,
            run_log_filename,
        },
        run_updates::{delete_history_run, open_run_log, update_history_run_status},
        HistoryStore,
    },
    models::{DestinationRootMode, EffectiveDestinationRoot, HistoryRunStatus},
    test_support::history::running_index_entry,
};

#[test]
fn builds_history_paths_under_app_data() {
    let app_data = Path::new("app-data").join("media-organizer");
    let history_root = history_root_from_app_data(&app_data);
    let runs_dir = history_runs_dir(&history_root);
    let index_path = history_index_path(&history_root);

    let expected_root = app_data.join("history");
    assert_eq!(history_root, expected_root);
    assert_eq!(runs_dir, expected_root.join("runs"));
    assert_eq!(index_path, expected_root.join("index.json"));
}

#[test]
fn generates_sortable_run_log_filename() {
    let started_at: DateTime<Utc> = Utc
        .with_ymd_and_hms(2026, 5, 24, 1, 2, 3)
        .single()
        .expect("valid utc time");

    let file_name = run_log_filename(started_at, "runabc123");
    assert_eq!(file_name, "2026-05-24T01-02-03Z_runabc123.jsonl");
}

#[test]
fn refuses_to_append_to_or_delete_a_log_outside_the_runs_folder() {
    let temp = tempdir().expect("temp dir");
    let history_root = temp.path().join("history");
    fs::create_dir_all(history_runs_dir(&history_root)).expect("create runs dir");
    let user_file = temp.path().join("photo.jsonl");
    fs::write(&user_file, b"user data").expect("write user file");
    let entry = running_index_entry(
        "run-1",
        "2026-05-24T01:02:03Z",
        &user_file.to_string_lossy(),
    );
    upsert_history_index_entry(&history_root, entry).expect("upsert index entry");

    assert!(open_run_log(&history_root, "run-1").is_err());
    assert!(update_history_run_status(&history_root, "run-1", HistoryRunStatus::Undone).is_err());
    assert!(delete_history_run(&history_root, "run-1").is_err());
    assert!(checked_run_log_path(&history_root, "../runs/x.jsonl").is_err());
    assert_eq!(fs::read(&user_file).expect("read user file"), b"user data");
}

#[test]
fn deletes_a_run_log_and_its_index_entry_once_the_run_is_done() {
    let temp = tempdir().expect("temp dir");
    let store = HistoryStore::in_app_data(temp.path());
    let root = EffectiveDestinationRoot {
        mode: DestinationRootMode::ProcessedFolder,
        root_path: temp.path().join("Processed").to_string_lossy().to_string(),
        custom_root_configured: false,
        force_processed_folder: false,
        warning: None,
    };
    let run = store
        .start_run(temp.path(), &root, Path::new(&root.root_path))
        .expect("start run");
    let log_path = Path::new(&run.run.history_file_path).to_path_buf();
    let run_id = run.run.run_id.clone();

    assert!(store.delete_run(&run_id).is_err());
    assert!(log_path.exists());
    drop(run);
    store.delete_run(&run_id).expect("delete run");

    assert!(!log_path.exists());
    assert!(store.list_runs().expect("list runs").is_empty());
}
