use std::fs::{self, OpenOptions};

use tempfile::tempdir;

use crate::{
    history::log_file::append_history_record,
    models::{
        DestinationRootMode, HistoryOperation, HistoryOperationKind, HistoryRecord, HistoryRun,
        HistoryRunCompletedRecord, HistoryRunStatus,
    },
    test_support::history::{history_operation, running_history_run},
};

#[test]
fn loads_legacy_history_run_without_destination_root_fields() {
    let raw = r#"{
        "run_id": "run-legacy",
        "history_version": 1,
        "app_version": "0.1.0",
        "selected_folder": "C:\\todo",
        "folder_label": "todo",
        "started_at": "2026-05-24T01:02:03Z",
        "completed_at": null,
        "run_status": "running",
        "history_file_path": "C:\\appdata\\history\\runs\\run-legacy.jsonl"
    }"#;

    let run: HistoryRun = serde_json::from_str(raw).expect("legacy run should deserialize safely");

    assert_eq!(run.destination_root_mode, None);
    assert_eq!(run.destination_root_path, None);
}

#[test]
fn serializes_history_run_destination_root_fields() {
    let run = HistoryRun {
        destination_root_mode: Some(DestinationRootMode::CustomRoot),
        destination_root_path: Some(r"D:\Media".to_string()),
        place_root_path: None,
        ..running_history_run(
            "run-custom-root",
            r"C:\appdata\history\runs\run-custom-root.jsonl",
        )
    };

    let serialized = serde_json::to_string(&run).expect("run should serialize");

    assert!(serialized.contains("\"destination_root_mode\":\"custom_root\""));
    assert!(serialized.contains("\"destination_root_path\":\"D:\\\\Media\""));
}

#[test]
fn appends_jsonl_history_records() {
    let temp = tempdir().expect("temp dir");
    let log_path = temp.path().join("history.jsonl");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("open log");

    let started_record = HistoryRecord::RunStarted {
        run: running_history_run("run-1", r"C:\appdata\history\runs\run-1.jsonl"),
    };
    let completed_record = HistoryRecord::RunCompleted(HistoryRunCompletedRecord {
        run_id: "run-1".to_string(),
        completed_at: "2026-05-24T01:05:00Z".to_string(),
        run_status: HistoryRunStatus::Completed,
    });
    let operation_record = HistoryRecord::Operation(HistoryOperation {
        destination_path: r"C:\todo\Processed\2026\05\24\photo.jpg".to_string(),
        expected_source_size: Some(1024),
        expected_destination_size: Some(1024),
        metadata_date: Some("2026-05-24T01:02:03Z".to_string()),
        ..history_operation(
            "run-1",
            "run-1-0000000000000001",
            "photo-1",
            HistoryOperationKind::Process,
        )
    });

    append_history_record(&mut file, &started_record).expect("append start");
    append_history_record(&mut file, &operation_record).expect("append operation");
    append_history_record(&mut file, &completed_record).expect("append complete");
    drop(file);

    let contents = fs::read_to_string(&log_path).expect("read log");
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains(r#""record_type":"run_started""#));
    assert!(lines[1].contains(r#""record_type":"operation""#));
    assert!(lines[2].contains(r#""record_type":"run_completed""#));
}
