use serde::{Deserialize, Serialize};

/// A trip or outing folder an earlier run filled, as a later suggestion continues it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct ContinuedPlacement {
    pub title: String,
    pub year: i32,
    pub start: String,
    pub end: String,
    /// The placed place folders. A place of the same name widens its day layout by them.
    pub places: Vec<ContinuedPlace>,
    /// Continues every folder of the suggestion's country in its year, at the suggestion's
    /// level, rather than one trip or outing. `title` is then the country folder.
    #[serde(default)]
    pub country_wide: bool,
    /// Month folders whose files of the country move up into the year folder.
    #[serde(default)]
    pub lifted: Vec<LiftedMonth>,
}

/// A month folder holding placed files of a country that move up to the year folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct LiftedMonth {
    pub month_folder: String,
    pub file_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct ContinuedPlace {
    pub name: String,
    pub start: String,
    pub end: String,
}
