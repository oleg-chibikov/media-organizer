use std::path::{Path, PathBuf};

use chrono::{Datelike, NaiveDateTime};

use crate::{
    models::{DayLayout, Placement},
    placements::place_folder,
};

/// The accepted place folder of one file: the folder of its placement under the root,
/// then day folders by `day_layout`.
#[derive(Debug, Clone)]
pub(crate) struct PlaceFolder {
    pub(super) day_layout: DayLayout,
    pub(super) placement: Placement,
}

impl PlaceFolder {
    pub(super) fn dir_under(&self, root: &Path, date: &NaiveDateTime) -> PathBuf {
        let dir = place_folder(root, &self.placement);
        match self.day_layout {
            DayLayout::Single => dir,
            DayLayout::MonthDay => dir.join(date.format("%m-%d").to_string()),
            DayLayout::MonthFolderDay => dir
                .join(date.month().to_string())
                .join(date.day().to_string()),
        }
    }

    /// Whether both put their files in the same place folder.
    pub(super) fn same_folder(&self, other: &Placement) -> bool {
        place_folder(Path::new(""), &self.placement) == place_folder(Path::new(""), other)
    }
}
