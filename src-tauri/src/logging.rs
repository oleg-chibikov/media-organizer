use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::models::{MoveResultStatus, MoveStrategy};

pub const PROCESSED_DIR_NAME: &str = "Processed";
pub const PROCESSING_LOG_FILENAME: &str = "processing-log.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingLogRecord {
    pub source_path: String,
    pub destination_path: String,
    pub metadata_date: Option<String>,
    pub original_name: String,
    pub move_strategy: MoveStrategy,
    pub status: MoveResultStatus,
    pub error: Option<String>,
    pub timestamp: String,
}

pub fn processing_log_path(selected_root: &Path) -> PathBuf {
    selected_root
        .join(PROCESSED_DIR_NAME)
        .join(PROCESSING_LOG_FILENAME)
}

pub fn ensure_processed_dir(selected_root: &Path) -> Result<PathBuf, String> {
    let processed_dir = selected_root.join(PROCESSED_DIR_NAME);
    fs::create_dir_all(&processed_dir).map_err(|error| {
        format!(
            "Failed to create Processed directory at '{}': {error}",
            processed_dir.display()
        )
    })?;
    Ok(processed_dir)
}

pub fn open_processing_log_for_append(selected_root: &Path) -> Result<File, String> {
    ensure_processed_dir(selected_root)?;
    let log_path = processing_log_path(selected_root);
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|error| {
            format!(
                "Failed to open processing log at '{}': {error}",
                log_path.display()
            )
        })
}

#[allow(clippy::too_many_arguments)]
pub fn build_processing_log_record(
    source_path: impl Into<String>,
    destination_path: impl Into<String>,
    metadata_date: Option<String>,
    original_name: impl Into<String>,
    move_strategy: MoveStrategy,
    status: MoveResultStatus,
    error: Option<String>,
    timestamp: DateTime<Local>,
) -> ProcessingLogRecord {
    ProcessingLogRecord {
        source_path: source_path.into(),
        destination_path: destination_path.into(),
        metadata_date,
        original_name: original_name.into(),
        move_strategy,
        status,
        error,
        timestamp: timestamp.to_rfc3339(),
    }
}

pub fn record_to_jsonl_line(record: &ProcessingLogRecord) -> Result<String, String> {
    serde_json::to_string(record)
        .map(|json| format!("{json}\n"))
        .map_err(|error| format!("Failed to serialize processing log record: {error}"))
}

pub fn append_record(file: &mut File, record: &ProcessingLogRecord) -> Result<(), String> {
    let line = record_to_jsonl_line(record)?;
    file.write_all(line.as_bytes())
        .map_err(|error| format!("Failed to append processing log record: {error}"))
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use std::fs;
    use tempfile::tempdir;

    use crate::models::{MoveResultStatus, MoveStrategy};

    use super::*;

    #[test]
    fn builds_processing_log_path_under_processed_folder() {
        let root = Path::new(r"C:\todo");
        let path = processing_log_path(root);
        let expected = Path::new(r"C:\todo\Processed\processing-log.jsonl");
        assert_eq!(path, expected);
    }

    #[test]
    fn serializes_jsonl_record_with_expected_fields() {
        let timestamp = DateTime::parse_from_rfc3339("2026-05-23T10:30:45+10:00")
            .expect("timestamp parse")
            .with_timezone(&Local);

        let record = build_processing_log_record(
            r"C:\todo\IMG_1001.jpg",
            r"C:\todo\Processed\2026\05\23\2026-05-23 10-00-00 (IMG_1001).jpg",
            Some("2026-05-23T10:00:00+10:00".to_string()),
            "IMG_1001",
            MoveStrategy::Rename,
            MoveResultStatus::Moved,
            None,
            timestamp,
        );

        let line = record_to_jsonl_line(&record).expect("serialize jsonl");

        assert!(line.ends_with('\n'));
        assert!(line.contains(r#""source_path":"C:\\todo\\IMG_1001.jpg""#));
        assert!(line.contains(r#""move_strategy":"rename""#));
        assert!(line.contains(r#""status":"moved""#));
        assert!(line.contains(r#""timestamp":"2026-05-23T10:30:45+10:00""#));
    }

    #[test]
    fn supports_error_records() {
        let timestamp = DateTime::parse_from_rfc3339("2026-05-23T10:31:00+10:00")
            .expect("timestamp parse")
            .with_timezone(&Local);

        let record = build_processing_log_record(
            r"C:\todo\missing.jpg",
            r"C:\todo\Processed\2026\05\23\missing.jpg",
            None,
            "missing",
            MoveStrategy::NotAttempted,
            MoveResultStatus::Skipped,
            Some("No source file record found for move plan.".to_string()),
            timestamp,
        );

        let line = record_to_jsonl_line(&record).expect("serialize jsonl");

        assert!(line.contains(r#""metadata_date":null"#));
        assert!(line.contains(r#""error":"No source file record found for move plan.""#));
        assert!(line.contains(r#""move_strategy":"not_attempted""#));
        assert!(line.contains(r#""status":"skipped""#));
    }

    #[test]
    fn creates_processed_directory_and_opens_log_for_append() {
        let temp = tempdir().expect("temp dir");
        let root = temp.path();

        let _file = open_processing_log_for_append(root).expect("open processing log");

        let processed = root.join(PROCESSED_DIR_NAME);
        let log_path = processed.join(PROCESSING_LOG_FILENAME);
        assert!(processed.exists());
        assert!(processed.is_dir());
        assert!(log_path.exists());
    }

    #[test]
    fn appends_multiple_jsonl_records_for_skipped_and_failed_results() {
        let temp = tempdir().expect("temp dir");
        let root = temp.path();
        let timestamp = DateTime::parse_from_rfc3339("2026-05-23T10:31:00+10:00")
            .expect("timestamp parse")
            .with_timezone(&Local);

        let skipped = build_processing_log_record(
            r"C:\todo\missing.jpg",
            r"C:\todo\Processed\2026\05\23\missing.jpg",
            None,
            "missing",
            MoveStrategy::NotAttempted,
            MoveResultStatus::Skipped,
            Some("No source file record found for move plan.".to_string()),
            timestamp,
        );
        let failed = build_processing_log_record(
            r"C:\todo\bad.jpg",
            r"C:\todo\Processed\2026\05\23\bad.jpg",
            Some("2026-05-23T10:00:00+10:00".to_string()),
            "bad",
            MoveStrategy::Rename,
            MoveResultStatus::Failed,
            Some("Atomic rename failed: access denied".to_string()),
            timestamp,
        );

        let mut file = open_processing_log_for_append(root).expect("open processing log");
        append_record(&mut file, &skipped).expect("append skipped");
        append_record(&mut file, &failed).expect("append failed");
        drop(file);

        let contents = fs::read_to_string(processing_log_path(root)).expect("read log");
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains(r#""status":"skipped""#));
        assert!(lines[1].contains(r#""status":"failed""#));
    }
}
