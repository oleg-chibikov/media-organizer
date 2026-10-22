use serde::{Deserialize, Serialize};

use super::{FileRecord, MetadataResult, Placement};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub enum ConflictStatus {
    None,
    SuffixApplied,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub enum PlanStatus {
    Ready,
    BlockedNoDate,
    BlockedConflict,
    /// The file already has its planned name in its planned folder.
    AlreadyNamed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct MovePlan {
    pub file_id: String,
    pub original_name: String,
    pub destination_dir: String,
    pub destination_filename: String,
    pub destination_path: String,
    pub conflict_status: ConflictStatus,
    pub plan_status: PlanStatus,
    #[serde(default)]
    pub placement: Option<Placement>,
}

/// A file an earlier run placed, planned into the new day layout of its folder.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PlacedMove {
    pub record: FileRecord,
    pub metadata: MetadataResult,
    pub plan: MovePlan,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PlanFileEvent {
    pub scan_id: u64,
    pub plan: MovePlan,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PlanErrorEvent {
    pub scan_id: u64,
    pub file_id: String,
    pub error: String,
}

/// One planning pass ended. The webview waits for `plan:settled` instead.
#[derive(Debug, Clone, Serialize)]
pub struct PlanFinishedEvent {
    pub scan_id: u64,
    pub planned: u64,
    pub errors: u64,
}

/// The scan's plans are gone, as after a new destination. New ones follow.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PlanClearedEvent {
    pub scan_id: u64,
}

/// Files earlier runs placed that join the scan, and the ones that leave it.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PlacedFilesEvent {
    pub scan_id: u64,
    pub added: Vec<PlacedMove>,
    pub dropped: Vec<String>,
}

/// Every file with a date is planned for the place folders the person chose.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PlanSettledEvent {
    pub scan_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
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
