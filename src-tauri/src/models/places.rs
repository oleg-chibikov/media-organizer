use serde::{Deserialize, Serialize};

/// How much of a place is home. Home parts stay out of the destination path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub enum HomeMatch {
    /// Abroad, or no home is known: country and place folders.
    Away,
    /// Home's country: a place folder only.
    Country,
    /// Home's city: no place folders.
    City,
}

/// The place of one file, as the file table shows it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct FilePlace {
    /// The matched place, else the city.
    pub place: String,
    pub country: String,
    pub home: HomeMatch,
}

impl FilePlace {
    /// The country folder above the place, outside home's country.
    pub fn country_folder(&self) -> Option<&str> {
        (self.home == HomeMatch::Away).then_some(self.country.as_str())
    }
}
