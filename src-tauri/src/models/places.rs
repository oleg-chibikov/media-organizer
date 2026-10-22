use serde::{Deserialize, Serialize};

/// The place folder of one file, as the file table shows it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct FilePlace {
    /// The matched place, else the city.
    pub place: String,
    /// The country folder above the place, for a place outside home's country.
    #[serde(default)]
    pub country: Option<String>,
}
