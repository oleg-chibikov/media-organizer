use std::path::{Path, PathBuf};

use chrono::{Datelike, NaiveDateTime};

use crate::models::{DayLayout, Placement};

/// The accepted place folder of one file: `<year>/<trip or place>[/<place>]`
/// under the root, then day folders by `day_layout`.
#[derive(Debug, Clone)]
pub(crate) struct PlaceFolder {
    pub(super) parts: Vec<String>,
    pub(super) day_layout: DayLayout,
    pub(super) placement: Placement,
}

impl PlaceFolder {
    pub(super) fn dir_under(&self, root: &Path, date: &NaiveDateTime) -> PathBuf {
        let dir: PathBuf = self
            .parts
            .iter()
            .fold(root.to_path_buf(), |dir, part| dir.join(part));
        match self.day_layout {
            DayLayout::Single => dir,
            DayLayout::MonthDay => dir.join(date.format("%m-%d").to_string()),
            DayLayout::MonthFolderDay => dir
                .join(date.month().to_string())
                .join(date.day().to_string()),
        }
    }
}
