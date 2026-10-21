//! Capture metadata: persistent ExifTool workers, JSON parsing and capture-date selection.

mod date_selection;
mod exiftool_json;
mod exiftool_path;
mod output_reader;
mod request_writer;
mod timestamp;
mod worker;
mod worker_batch;
mod worker_manager;

#[cfg(test)]
mod tests;

use crate::models::DateKind;

pub const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 20_000;

pub(crate) use date_selection::select_metadata_date;
pub(crate) use worker_manager::ExifToolWorkerManager;

#[derive(Debug, Clone)]
pub struct RawMetadataDates {
    pub sub_sec_date_time_original: Option<String>,
    pub date_time_original: Option<String>,
    pub create_date: Option<String>,
    pub modify_date: Option<String>,
    pub media_create_date: Option<String>,
    pub track_create_date: Option<String>,
    pub file_modify_date: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SelectedMetadataDate {
    pub chosen_date: String,
    pub chosen_date_source: String,
    pub date_kind: DateKind,
    pub raw_metadata_date: String,
}

#[derive(Debug, Clone)]
pub struct MetadataBatchReadOutcome {
    pub raw_dates: Vec<RawMetadataDates>,
    pub did_restart: bool,
}
