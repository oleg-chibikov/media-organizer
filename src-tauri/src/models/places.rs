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
    /// The placed trip or outing this one continues. Its title and year win.
    #[serde(default)]
    pub continues: Option<ContinuedPlacement>,
}

/// A trip or outing folder an earlier run filled, as a later suggestion continues it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuedPlacement {
    pub title: String,
    pub year: i32,
    pub start: String,
    pub end: String,
    /// The placed place folders. A place of the same name widens its day layout by them.
    pub places: Vec<ContinuedPlace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuedPlace {
    pub name: String,
    pub start: String,
    pub end: String,
}

/// The place folder a planned file goes to, so a finished run can record it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placement {
    pub kind: SuggestionKind,
    pub year: i32,
    /// The trip folder, or the outing's place folder.
    pub title: String,
    /// The place folder inside a trip, or the outing's place. `None` for trip root files.
    pub place: Option<String>,
}
