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
    pub kind: FileKind,
    pub scan_status: ScanStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateKind {
    TimezoneAware,
    TimezoneUnknown,
    Fallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataStatus {
    Pending,
    Ready,
    NoDate,
    Error,
    ExiftoolRestarted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataResult {
    pub file_id: String,
    pub chosen_date: Option<String>,
    pub chosen_date_source: Option<String>,
    pub date_kind: Option<DateKind>,
    pub raw_metadata_date: Option<String>,
    pub metadata_status: MetadataStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStatus {
    None,
    SuffixApplied,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Ready,
    BlockedNoDate,
    BlockedConflict,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovePlan {
    pub file_id: String,
    pub original_name: String,
    pub destination_dir: String,
    pub destination_filename: String,
    pub destination_path: String,
    pub conflict_status: ConflictStatus,
    pub plan_status: PlanStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveStrategy {
    Rename,
    CopyRemove,
    NotAttempted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveResultStatus {
    Moved,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveResult {
    pub file_id: String,
    pub attempted_source_path: String,
    pub attempted_destination_path: String,
    pub move_strategy: MoveStrategy,
    pub result_status: MoveResultStatus,
    pub error: Option<String>,
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
