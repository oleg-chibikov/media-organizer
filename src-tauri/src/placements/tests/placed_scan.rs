use std::path::{Path, PathBuf};

use tempfile::{tempdir, TempDir};

use crate::{
    models::{PlaceSuggestion, SuggestionKind},
    placements::{continue_placed_suggestions, PlacementStore},
    settings::SettingsStore,
    test_support::placements::{detected, moved_into, place_group, place_on_disk, placement},
};

/// A scan of `<temp>/todo`, whose destination root is `<temp>/todo/Processed`.
pub(super) struct Scan {
    pub(super) temp: TempDir,
    store: PlacementStore,
}

impl Scan {
    pub(super) fn new() -> Self {
        let temp = tempdir().expect("temp dir");
        let store = PlacementStore::in_app_data(&temp.path().join("app-data"));
        Self { temp, store }
    }

    pub(super) fn selected(&self) -> PathBuf {
        self.temp.path().join("todo")
    }

    pub(super) fn store(&self) -> PlacementStore {
        self.store.clone()
    }

    pub(super) fn root(&self) -> PathBuf {
        self.selected().join("Processed")
    }

    /// Places one file per `(place, relative path, date)` under `root`.
    pub(super) fn place(
        &self,
        root: &Path,
        kind: SuggestionKind,
        year: i32,
        title: &str,
        files: &[(&str, &str, &str)],
    ) {
        let moves: Vec<_> = files
            .iter()
            .map(|(place, path, date)| {
                moved_into(root, &placement(kind, year, title, Some(place)), path, date)
            })
            .collect();
        place_on_disk(&self.store, root, &moves);
    }

    pub(super) fn continue_all(&self, suggestions: Vec<PlaceSuggestion>) -> Vec<PlaceSuggestion> {
        let settings = SettingsStore::in_app_data(&self.temp.path().join("app-data"));
        continue_placed_suggestions(
            &self.store,
            &settings,
            &self.selected(),
            &[],
            &[],
            suggestions,
        )
        .expect("continued")
    }
}

/// October in New York and Vancouver, placed under the 2025 "United States" folder.
pub(super) fn october_scan() -> Scan {
    let scan = Scan::new();
    scan.place(
        &scan.root(),
        SuggestionKind::Trip,
        2025,
        "United States",
        &[
            (
                "New York",
                "2025/United States/New York/10-28/a.jpg",
                "2025-10-28 10:00:00",
            ),
            (
                "New York",
                "2025/United States/New York/10-30/b.jpg",
                "2025-10-30 10:00:00",
            ),
            (
                "Vancouver",
                "2025/United States/Vancouver/c.jpg",
                "2025-10-31 18:00:00",
            ),
        ],
    );
    scan
}

pub(super) fn trip(title: &str, start: &str, end: &str) -> PlaceSuggestion {
    let places = vec![
        place_group("vancouver", start, start, &["v1"]),
        place_group("Toronto", end, end, &["t1"]),
    ];
    detected(SuggestionKind::Trip, title, places)
}
