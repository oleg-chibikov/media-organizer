use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use tempfile::tempdir;

use crate::{
    history::{
        index::upsert_history_index_entry,
        log_file::append_history_record,
        paths::history_runs_dir,
        queries::{get_history_run_from_root, list_history_runs_from_root},
    },
    models::{
        HistoryIndexEntry, HistoryOperation, HistoryOperationKind, HistoryRecord, HistoryRunStatus,
    },
    test_support::history::{history_operation, running_history_run, running_index_entry},
};

/// Creates an empty run log and an index entry that still claims undo/redo is available.
fn indexed_run_log(history_root: &Path, run_id: &str) -> (PathBuf, File) {
    let runs_dir = history_runs_dir(history_root);
    fs::create_dir_all(&runs_dir).expect("create runs dir");
    let log_path = runs_dir.join(format!("2026-05-24T01-02-03Z_{run_id}.jsonl"));
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .expect("open run log");

    let log_path_value = log_path.to_string_lossy().to_string();
    upsert_history_index_entry(
        history_root,
        HistoryIndexEntry {
            undo_available: true,
            redo_available: true,
            ..running_index_entry(run_id, "2026-05-24T01:02:03Z", &log_path_value)
        },
    )
    .expect("write index entry");

    (log_path, file)
}

#[test]
fn recovers_partial_trailing_history_record_as_partial_and_unsafe() {
    let temp = tempdir().expect("temp dir");
    let history_root = temp.path().join("history");
    let (log_path, mut file) = indexed_run_log(&history_root, "run-1");

    let run = running_history_run("run-1", &log_path.to_string_lossy());
    append_history_record(&mut file, &HistoryRecord::RunStarted { run: run.clone() })
        .expect("write start record");
    append_history_record(
        &mut file,
        &HistoryRecord::Operation(HistoryOperation {
            source_path: r"C:\todo\in.jpg".to_string(),
            destination_path: r"C:\todo\Processed\out.jpg".to_string(),
            expected_source_size: Some(12),
            expected_destination_size: Some(12),
            metadata_date: Some("2026-05-24T01:00:00Z".to_string()),
            original_name: "in".to_string(),
            ..history_operation(
                "run-1",
                "run-1-0000000000000001",
                "file-1",
                HistoryOperationKind::Process,
            )
        }),
    )
    .expect("write operation record");
    file.write_all(br#"{"record_type":"run_completed""#)
        .expect("write partial trailing record");
    file.flush().expect("flush partial record");

    let runs = list_history_runs_from_root(&history_root).expect("list runs");
    assert_eq!(runs.len(), 1);
    assert!(matches!(runs[0].run_status, HistoryRunStatus::Partial));
    assert_eq!(runs[0].files_migrated, 1);
    assert!(!runs[0].undo_available);
    assert!(!runs[0].redo_available);

    let details = get_history_run_from_root(&history_root, "run-1")
        .expect("load run details")
        .expect("run details should exist");
    assert!(matches!(details.run.run_status, HistoryRunStatus::Partial));
    assert!(details.unsafe_for_undo_redo);
    assert_eq!(details.operations.len(), 1);
}

#[test]
fn marks_malformed_history_record_as_corrupt_and_unsafe() {
    let temp = tempdir().expect("temp dir");
    let history_root = temp.path().join("history");
    let (log_path, mut file) = indexed_run_log(&history_root, "run-2");

    append_history_record(
        &mut file,
        &HistoryRecord::RunStarted {
            run: running_history_run("run-2", &log_path.to_string_lossy()),
        },
    )
    .expect("write start record");
    file.write_all(b"not-json\n")
        .expect("write malformed record");
    file.flush().expect("flush malformed record");

    let runs = list_history_runs_from_root(&history_root).expect("list runs");
    assert_eq!(runs.len(), 1);
    assert!(matches!(runs[0].run_status, HistoryRunStatus::Corrupt));
    assert_eq!(runs[0].files_migrated, 0);
    assert!(!runs[0].undo_available);
    assert!(!runs[0].redo_available);

    let details = get_history_run_from_root(&history_root, "run-2")
        .expect("load run details")
        .expect("run details should exist");
    assert!(matches!(details.run.run_status, HistoryRunStatus::Corrupt));
    assert!(details.unsafe_for_undo_redo);
    assert!(details.recovery_error.is_some());
}
