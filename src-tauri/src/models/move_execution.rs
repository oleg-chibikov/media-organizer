use serde::{Deserialize, Serialize};

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
pub struct MoveFileEvent {
    pub result: MoveResult,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveProgressPhase {
    Starting,
    Preparing,
    Moving,
    Finalizing,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoveProgressEvent {
    pub phase: MoveProgressPhase,
    pub file_id: Option<String>,
    pub message: String,
    pub completed: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoveFinishedEvent {
    pub moved: u64,
    pub copied: u64,
    pub skipped: u64,
    pub failed: u64,
    pub results: Vec<MoveResult>,
}
