mod recorded_roots;
mod redo;
mod undo;

use std::path::{Path, PathBuf};

use crate::{
    models::{HistoryOperation, HistoryOperationKind},
    test_support::history::history_operation,
};

/// Original source and processed destination paths of one photo inside `root`.
fn photo_paths(root: &Path) -> (PathBuf, PathBuf) {
    let original_source = root.join("todo").join("photo.jpg");
    let processed_destination = root
        .join("todo")
        .join("Processed")
        .join("2026")
        .join("05")
        .join("24")
        .join("photo.jpg");
    (original_source, processed_destination)
}

/// Succeeded rename record for an 11-byte file moved from `source` to `destination`.
fn recorded_rename(
    operation_id: &str,
    operation_kind: HistoryOperationKind,
    source: &Path,
    destination: &Path,
    timestamp: &str,
) -> HistoryOperation {
    HistoryOperation {
        source_path: source.to_string_lossy().to_string(),
        destination_path: destination.to_string_lossy().to_string(),
        expected_source_size: Some(11),
        expected_destination_size: Some(11),
        metadata_date: Some("2026-05-24T01:02:03Z".to_string()),
        timestamp: timestamp.to_string(),
        ..history_operation("run-1", operation_id, "file-1", operation_kind)
    }
}
