use std::fs;

use chrono::{Duration, TimeZone, Utc};
use tempfile::tempdir;

use crate::{
    history::{
        index::HistoryIndexFile, index_updates::upsert_history_index_entry,
        paths::history_index_path, MAX_INDEX_RUNS,
    },
    test_support::history::running_index_entry,
};

#[test]
fn keeps_index_sorted_and_bounded() {
    let temp = tempdir().expect("temp dir");
    let history_root = temp.path().join("history");
    let base_started_at = Utc
        .with_ymd_and_hms(2026, 5, 24, 0, 0, 0)
        .single()
        .expect("valid utc time");

    for value in 0..(MAX_INDEX_RUNS + 5) {
        let started_at = base_started_at + Duration::seconds(value as i64);
        let entry = running_index_entry(
            &format!("run-{value}"),
            &started_at.to_rfc3339(),
            &format!(r"C:\appdata\history\runs\run-{value}.jsonl"),
        );
        upsert_history_index_entry(&history_root, entry).expect("upsert index entry");
    }

    let index_path = history_index_path(&history_root);
    let raw = fs::read_to_string(index_path).expect("read index");
    let index: HistoryIndexFile = serde_json::from_str(&raw).expect("parse index");

    assert_eq!(index.runs.len(), MAX_INDEX_RUNS);
    assert_eq!(index.runs[0].run_id, format!("run-{}", MAX_INDEX_RUNS + 4));
    assert_eq!(index.runs[MAX_INDEX_RUNS - 1].run_id, "run-5");
}
