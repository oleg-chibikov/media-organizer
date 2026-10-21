use std::path::Path;

use chrono::{DateTime, TimeZone, Utc};

use crate::history::paths::{
    history_index_path, history_root_from_app_data, history_runs_dir, run_log_filename,
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
