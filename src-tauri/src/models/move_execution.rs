use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub enum MoveStrategy {
    Rename,
    CopyKeepSource,
    NotAttempted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub enum MoveResultStatus {
    Moved,
    Copied,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct MoveResult {
    pub file_id: String,
    pub attempted_source_path: String,
    pub attempted_destination_path: String,
    pub move_strategy: MoveStrategy,
    pub result_status: MoveResultStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct MoveFileEvent {
    pub result: MoveResult,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub enum MoveProgressPhase {
    Starting,
    Preparing,
    Moving,
    Finalizing,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct MoveProgressEvent {
    pub phase: MoveProgressPhase,
    pub file_id: Option<String>,
    pub message: String,
    pub completed: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct MoveFinishedEvent {
    pub moved: u64,
    pub copied: u64,
    pub skipped: u64,
    pub failed: u64,
    pub results: Vec<MoveResult>,
}

/// The confirmed run moved every ready file and the scan has nothing left to check.
/// Empty source folders are gone by now.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct MovesDoneEvent {
    pub scan_id: u64,
    /// The picked folder itself was empty and got removed.
    pub removed_selected_folder: bool,
    /// Why the run stopped early, or why the source folders stayed.
    pub error: Option<String>,
}
