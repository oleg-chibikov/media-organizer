use serde::{Deserialize, Serialize};

use super::FilePlace;

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
    /// The place folder the file goes to, under the place folder root.
    #[serde(default)]
    pub place: Option<FilePlace>,
}

/// A file the planner couldn't plan, with the reason.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PlanFileError {
    pub file_id: String,
    pub error: String,
}

/// The plans and errors of one planning pass, in one event instead of one per file.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PlanFilesEvent {
    pub scan_id: u64,
    pub plans: Vec<MovePlan>,
    pub errors: Vec<PlanFileError>,
}

/// The scan's plans are gone, as after a new destination. New ones follow.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PlanClearedEvent {
    pub scan_id: u64,
}

/// Every file with a date is planned for the place folders found.
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
