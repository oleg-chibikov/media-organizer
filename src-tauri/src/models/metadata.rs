use serde::{Deserialize, Serialize};

use super::{FileKind, GpsPoint};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub enum DateKind {
    TimezoneAware,
    TimezoneUnknown,
    Fallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub enum MetadataStatus {
    Pending,
    Ready,
    NoDate,
    Error,
    ExiftoolRestarted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct MetadataResult {
    pub file_id: String,
    pub chosen_date: Option<String>,
    pub chosen_date_source: Option<String>,
    pub date_kind: Option<DateKind>,
    pub raw_metadata_date: Option<String>,
    pub device_name: Option<String>,
    pub gps: Option<GpsPoint>,
    pub metadata_status: MetadataStatus,
    pub error: Option<String>,
}

/// One file for a metadata reader. It stays inside the backend.
#[derive(Debug, Clone)]
pub struct MetadataReadRequest {
    pub file_id: String,
    pub source_path: String,
    pub file_kind: FileKind,
}

/// Dates the backend read for files of one scan, sent in groups as reads finish.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct MetadataResultsEvent {
    pub scan_id: u64,
    pub results: Vec<MetadataResult>,
}

/// Every photo and video of the scan has a result, or the scan was cancelled.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct MetadataFinishedEvent {
    pub scan_id: u64,
    pub cancelled: bool,
}
