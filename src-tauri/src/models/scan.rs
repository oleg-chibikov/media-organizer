use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub enum FileKind {
    Photo,
    Video,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub enum ScanStatus {
    Discovered,
    SkippedProcessed,
    SkippedUnsupported,
    ScanError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
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
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct ScanStartedEvent {
    pub root: String,
    /// Tells this scan's metadata events from those of the scan it replaced.
    pub scan_id: u64,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct ScanFilesEvent {
    pub records: Vec<FileRecord>,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct ScanErrorEvent {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct ScanFinishedEvent {
    pub discovered: u64,
    pub skipped: u64,
    pub errors: u64,
    pub cancelled: bool,
}
