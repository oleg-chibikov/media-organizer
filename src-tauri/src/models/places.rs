use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionKind {
    /// Far from home, possibly across several places and countries.
    Trip,
    /// A visit to one notable place, near home or with no known home.
    Outing,
}

/// How a place folder splits its files by day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DayLayout {
    /// One day: files sit in the place folder itself.
    Single,
    /// Up to 31 days: `MM-DD` folders.
    MonthDay,
    /// Longer: `<month>/<day>` folders.
    MonthFolderDay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceGroup {
    pub name: String,
    pub place_kind: String,
    pub day_layout: DayLayout,
    pub start: String,
    pub end: String,
    pub file_ids: Vec<String>,
}

/// A suggested place folder. Dates use the `chosen_date` format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceSuggestion {
    pub id: String,
    pub kind: SuggestionKind,
    pub year: i32,
    pub title: String,
    pub start: String,
    pub end: String,
    pub file_count: u32,
    pub places: Vec<PlaceGroup>,
    /// Trip files that belong to no place folder.
    pub root_file_ids: Vec<String>,
}
