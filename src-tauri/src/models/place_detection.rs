use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::FilePlace;

/// What `detect_file_places` found for one scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PlaceDetection {
    /// The place folder of each photo or video that has one, by file id.
    pub file_places: HashMap<String, FilePlace>,
    pub status: DetectionStatus,
}

/// Whether detection found places, or why it found none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub enum DetectionStatus {
    Found,
    /// No photo or video with a date has GPS.
    NoGps,
}
