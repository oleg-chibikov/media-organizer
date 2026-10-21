//! Capture metadata: persistent ExifTool workers, JSON parsing and capture-date selection.

mod argfile_path;
mod date_selection;
mod exiftool_json;
mod exiftool_path;
mod gps;
mod output_reader;
mod request_writer;
mod result_alignment;
mod timestamp;
mod worker;
mod worker_batch;
mod worker_manager;

#[cfg(test)]
mod tests;

use crate::models::{DateKind, GpsPoint};

pub const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 20_000;

pub(crate) use argfile_path::exiftool_path_argument;
pub(crate) use date_selection::select_metadata_date;
pub(crate) use worker_manager::ExifToolWorkerManager;

#[derive(Debug, Clone)]
pub struct RawMetadataDates {
    pub source_file: Option<String>,
    pub sub_sec_date_time_original: Option<String>,
    pub date_time_original: Option<String>,
    pub create_date: Option<String>,
    pub modify_date: Option<String>,
    pub media_create_date: Option<String>,
    pub track_create_date: Option<String>,
    pub file_modify_date: Option<String>,
    pub device_name: Option<String>,
    pub gps: Option<GpsPoint>,
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
    /// One slot per requested path; `None` when ExifTool returned no result for it.
    pub raw_dates: Vec<Option<RawMetadataDates>>,
    pub did_restart: bool,
}
