use serde::{Deserialize, Serialize};

/// Where a file was captured, in signed decimal degrees. South and west are negative.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GpsPoint {
    pub latitude: f64,
    pub longitude: f64,
}
