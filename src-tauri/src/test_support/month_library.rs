use std::{
    fs,
    path::{Path, PathBuf},
};

use tempfile::{tempdir, TempDir};

use crate::{
    models::{PlaceSuggestion, Placement, SuggestionKind},
    placements::{continue_placed_suggestions, PlacementStore},
    settings::{set_root_media_folder, SettingsStore},
    test_support::placements::{moved_into, place_on_disk, placement},
};

pub(crate) const USA: &str = "USA";

/// A library sorted by date, `<temp>/Library/<year>/<month>/<day>`, set as the custom root.
pub(crate) struct MonthLibrary {
    temp: TempDir,
    pub(crate) store: PlacementStore,
    pub(crate) settings: SettingsStore,
}

impl MonthLibrary {
    pub(crate) fn new() -> Self {
        let temp = tempdir().expect("temp dir");
        let app_data = temp.path().join("app-data");
        let (store, settings) = (
            PlacementStore::in_app_data(&app_data),
            SettingsStore::in_app_data(&app_data),
        );
        let root = temp.path().join("Library");
        fs::create_dir_all(&root).expect("library");
        let root_text = root.to_string_lossy().to_string();
        settings
            .update(|current| set_root_media_folder(current, Some(root_text)))
            .expect("custom root");
        Self {
            temp,
            store,
            settings,
        }
    }

    pub(crate) fn app_data(&self) -> PathBuf {
        self.temp.path().join("app-data")
    }

    pub(crate) fn root(&self) -> PathBuf {
        self.temp.path().join("Library")
    }

    /// `<root>/2026/<name>`, created on disk.
    pub(crate) fn month(&self, name: &str) -> PathBuf {
        let folder = self.root().join("2026").join(name);
        fs::create_dir_all(&folder).expect("month folder");
        folder
    }

    /// A USA trip a May scan placed in `2026/5/USA`: two Yosemite days and a root file.
    pub(crate) fn place_may_usa_trip(&self) {
        let usa = |place: Option<&str>| Placement {
            country: Some(USA.to_owned()),
            month_folder: Some("5".to_owned()),
            ..placement(SuggestionKind::Trip, 2026, USA, place)
        };
        let root = self.root();
        place_on_disk(
            &self.store,
            &root,
            &[
                moved_into(
                    &root,
                    &usa(Some("Yosemite")),
                    "2026/5/USA/Yosemite/05-03/a.jpg",
                    "2026-05-03 10:00:00",
                ),
                moved_into(
                    &root,
                    &usa(Some("Yosemite")),
                    "2026/5/USA/Yosemite/05-04/b.jpg",
                    "2026-05-04 10:00:00",
                ),
                moved_into(
                    &root,
                    &usa(None),
                    "2026/5/USA/05-05/r.jpg",
                    "2026-05-05 10:00:00",
                ),
            ],
        );
    }

    /// `suggestions` as detection returns them for a scan of `selected`.
    pub(crate) fn detect(
        &self,
        selected: &Path,
        suggestions: Vec<PlaceSuggestion>,
    ) -> Vec<PlaceSuggestion> {
        continue_placed_suggestions(&self.store, &self.settings, selected, suggestions)
            .expect("continued")
    }
}
