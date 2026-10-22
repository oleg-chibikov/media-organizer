use serde::{Deserialize, Serialize};

/// How the saved home was chosen.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HomeSource {
    /// Inferred from a large scan.
    Learned,
    /// Picked by the person in settings.
    Manual,
    /// Read from the device's current location.
    Device,
    /// Guessed from the system time zone, the weakest source.
    TimeZone,
}

/// The city trips are measured from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HomeLocation {
    pub name: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub source: HomeSource,
}

/// A city from the offline place table that can become home.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HomeCandidate {
    pub name: String,
    /// State or region, to tell same-named cities apart.
    pub region: Option<String>,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}
