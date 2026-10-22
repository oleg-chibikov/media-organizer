use serde::{Deserialize, Serialize};

use super::{FileRecord, MetadataResult, Placement};

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
    /// The file already has its planned name in its planned folder.
    AlreadyNamed,
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
    #[serde(default)]
    pub placement: Option<Placement>,
}

/// A file an earlier run placed, planned into the new day layout of its folder.
#[derive(Debug, Clone, Serialize)]
pub struct PlacedMove {
    pub record: FileRecord,
    pub metadata: MetadataResult,
    pub plan: MovePlan,
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
