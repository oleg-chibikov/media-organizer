use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    Photo,
    Video,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Discovered,
    SkippedProcessed,
    SkippedUnsupported,
    ScanError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: String,
    pub source_path: String,
    pub relative_path: String,
    pub extension: String,
    pub original_name: String,
    pub kind: FileKind,
    pub scan_status: ScanStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanStartedEvent {
    pub root: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanFileEvent {
    pub record: FileRecord,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanErrorEvent {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanFinishedEvent {
    pub discovered: u64,
    pub skipped: u64,
    pub errors: u64,
    pub cancelled: bool,
}
