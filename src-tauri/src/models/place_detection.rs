use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::PlaceSuggestion;

/// What `detect_place_suggestions` found for one scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceDetection {
    pub suggestions: Vec<PlaceSuggestion>,
    /// A short place label per geotagged photo or video, by file id.
    pub file_places: HashMap<String, String>,
    pub status: DetectionStatus,
}

/// Why detection found suggestions, or the first reason it found none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DetectionStatus {
    Found,
    /// No photo or video with a date has GPS.
    NoGps,
    /// No saved home and the files don't show one, so trips can't be measured.
    HomeUnknown,
    /// Every geotagged file is within trip distance of `home`, the home city.
    AllNearHome {
        home: String,
    },
    /// Some files are away or at a notable place, but too few to make a folder.
    TooFewGeotagged,
}
