use serde::{Deserialize, Serialize};

use super::FileKind;

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
