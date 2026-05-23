use std::path::{Path, PathBuf};

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

#[cfg(test)]
mod tests {
    use chrono::DateTime;

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
}
