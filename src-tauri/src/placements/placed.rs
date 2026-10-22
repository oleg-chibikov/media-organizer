use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    models::{Placement, SuggestionKind},
    path_safety::paths_equal_case_insensitive,
};

/// A trip or outing folder a run filled. Names are the folder names it used.
/// Dates use the `chosen_date` format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PlacedSuggestion {
    /// The absolute destination root the folder sits in.
    pub(crate) root: String,
    pub(crate) kind: SuggestionKind,
    pub(crate) year: i32,
    pub(crate) title: String,
    pub(crate) start: String,
    pub(crate) end: String,
    #[serde(default)]
    pub(crate) places: Vec<PlacedPlace>,
    /// Trip files in no place folder.
    #[serde(default)]
    pub(crate) root_files: Vec<PlacedFile>,
    /// The country folder outside home's country, as in `Placement::country`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) country: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PlacedPlace {
    pub(crate) name: String,
    pub(crate) start: String,
    pub(crate) end: String,
    #[serde(default)]
    pub(crate) files: Vec<PlacedFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PlacedFile {
    /// From the root, with `/` between folders.
    pub(crate) relative_path: String,
    pub(crate) chosen_date: String,
}

impl PlacedSuggestion {
    /// The trip folder, or the outing's place folder.
    pub(crate) fn folder(&self) -> PathBuf {
        top_folder(
            Path::new(&self.root),
            self.kind,
            self.year,
            self.country.as_deref(),
            &self.title,
        )
    }

    pub(crate) fn is_in(&self, root: &Path) -> bool {
        paths_equal_case_insensitive(Path::new(&self.root), root)
    }

    /// The same folder as `placement` under `root`.
    pub(crate) fn holds(&self, root: &Path, placement: &Placement) -> bool {
        self.is_in(root)
            && self.kind == placement.kind
            && self.year == placement.year
            && self.title == placement.title
            && self.country == placement.country
    }

    /// The files of one place folder, or the trip root files for `None`.
    pub(crate) fn files_in(&self, place: Option<&str>) -> &[PlacedFile] {
        match place {
            Some(name) => self
                .places
                .iter()
                .find(|placed| placed.name == name)
                .map_or(&[], |placed| placed.files.as_slice()),
            None => &self.root_files,
        }
    }
}

/// `<root>/<year>/<title>`, with an outing abroad inside `<country>`. A trip abroad is
/// titled after its country already.
pub(crate) fn top_folder(
    root: &Path,
    kind: SuggestionKind,
    year: i32,
    country: Option<&str>,
    title: &str,
) -> PathBuf {
    let year_folder = root.join(year.to_string());
    match (kind, country) {
        (SuggestionKind::Outing, Some(country)) => year_folder.join(country).join(title),
        _ => year_folder.join(title),
    }
}
