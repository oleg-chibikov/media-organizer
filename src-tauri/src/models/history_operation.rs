use serde::{Deserialize, Serialize};

use super::MoveStrategy;

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

/// Payload of both `history:undo-progress` and `history:redo-progress`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryReplayProgressEvent {
    pub run_id: String,
    pub total_operations: u64,
    pub completed_operations: u64,
    pub operation: HistoryOperation,
}
