use serde::{Deserialize, Serialize};

use super::{DestinationRootMode, HistoryOperation};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRun {
    pub run_id: String,
    pub history_version: u32,
    pub app_version: String,
    pub selected_folder: String,
    pub folder_label: String,
    #[serde(default)]
    pub destination_root_mode: Option<DestinationRootMode>,
    #[serde(default)]
    pub destination_root_path: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub run_status: HistoryRunStatus,
    pub history_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryIndexEntry {
    pub run_id: String,
    pub folder_label: String,
    pub selected_folder: String,
    #[serde(default)]
    pub destination_root_path: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub run_status: HistoryRunStatus,
    #[serde(default)]
    pub files_migrated: u64,
    pub undo_available: bool,
    pub redo_available: bool,
    pub history_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "record_type", rename_all = "snake_case")]
pub enum HistoryRecord {
    RunStarted { run: HistoryRun },
    Operation(HistoryOperation),
    RunCompleted(HistoryRunCompletedRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRunCompletedRecord {
    pub run_id: String,
    pub completed_at: String,
    pub run_status: HistoryRunStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRunDetails {
    pub run: HistoryRun,
    pub operations: Vec<HistoryOperation>,
    pub unsafe_for_undo_redo: bool,
    pub recovery_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryIndexUpdatedEvent {
    pub run_id: Option<String>,
}
