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
    pub device_name: Option<String>,
    pub metadata_status: MetadataStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataReadRequest {
    pub file_id: String,
    pub source_path: String,
    pub file_kind: FileKind,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessingConfig {
    pub metadata_concurrency: usize,
    pub metadata_queue_capacity: usize,
    pub metadata_timeout_ms: u64,
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
#[allow(dead_code)]
pub enum MoveStrategy {
    Rename,
    CopyKeepSource,
    NotAttempted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum MoveResultStatus {
    Moved,
    Copied,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
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

#[derive(Debug, Clone, Serialize)]
pub struct PlanFileEvent {
    pub plan: MovePlan,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanErrorEvent {
    pub file_id: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanFinishedEvent {
    pub planned: u64,
    pub errors: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoveFileEvent {
    pub result: MoveResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoveFinishedEvent {
    pub moved: u64,
    pub copied: u64,
    pub skipped: u64,
    pub failed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveConfirmationSummary {
    pub total_plans: u64,
    pub ready_operations: u64,
    pub skipped_operations: u64,
    pub blocked_no_date: u64,
    pub blocked_conflict: u64,
    pub plan_errors: u64,
    pub suffix_applied: u64,
    pub fallback_warnings: u64,
    pub photos_ready: u64,
    pub videos_ready: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum HistoryRunStatus {
    Running,
    Completed,
    CompletedWithErrors,
    Undone,
    PartiallyUndone,
    Redone,
    Partial,
    Corrupt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum HistoryOperationKind {
    Process,
    Undo,
    Redo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum HistoryOperationStatus {
    Succeeded,
    Skipped,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct HistoryRun {
    pub run_id: String,
    pub history_version: u32,
    pub app_version: String,
    pub selected_folder: String,
    pub folder_label: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub run_status: HistoryRunStatus,
    pub history_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct HistoryOperation {
    pub operation_id: String,
    pub run_id: String,
    pub file_id: String,
    pub operation_kind: HistoryOperationKind,
    pub source_path: String,
    pub destination_path: String,
    pub expected_source_size: Option<u64>,
    pub expected_destination_size: Option<u64>,
    pub metadata_date: Option<String>,
    pub original_name: String,
    pub move_strategy: MoveStrategy,
    pub status: HistoryOperationStatus,
    pub error: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct HistoryIndexEntry {
    pub run_id: String,
    pub folder_label: String,
    pub selected_folder: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub run_status: HistoryRunStatus,
    pub undo_available: bool,
    pub redo_available: bool,
    pub history_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "record_type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum HistoryRecord {
    RunStarted { run: HistoryRun },
    Operation(HistoryOperation),
    RunCompleted(HistoryRunCompletedRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct HistoryRunCompletedRecord {
    pub run_id: String,
    pub completed_at: String,
    pub run_status: HistoryRunStatus,
}
