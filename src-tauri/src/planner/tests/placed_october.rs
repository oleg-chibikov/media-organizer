use std::{fs, path::PathBuf};

use tempfile::{tempdir, TempDir};

use crate::{
    models::{PlaceSuggestion, PlacedMove, SuggestionKind},
    placements::{continue_placed_suggestions, PlacementStore},
    planner::plan_continued_moves,
    settings::SettingsStore,
    test_support::{
        media::dated_metadata,
        placements::{detected, moved_into, place_group, place_on_disk, placement},
    },
};

pub(super) const USA: &str = "United States";

/// Files of the October trip: place, path under `Processed` and date taken.
const OCTOBER_FILES: [(Option<&str>, &str, &str); 4] = [
    (
        Some("New York"),
        "2025/United States/New York/10-28/a.jpg",
        "2025-10-28 10:00:00",
    ),
    (
        Some("New York"),
        "2025/United States/New York/10-30/b.jpg",
        "2025-10-30 10:00:00",
    ),
    (
        None,
        "2025/United States/10-30/r.jpg",
        "2025-10-30 20:00:00",
    ),
    (
        Some("Vancouver"),
        "2025/United States/Vancouver/c.jpg",
        "2025-10-31 18:00:00",
    ),
];

/// `<temp>/todo` with an October trip placed in `todo/Processed`: two New York days,
/// one root file and one Vancouver file with no day folder.
pub(super) struct PlacedOctober {
    temp: TempDir,
    store: PlacementStore,
    settings: SettingsStore,
}

impl PlacedOctober {
    pub(super) fn new() -> Self {
        let temp = tempdir().expect("temp dir");
        let app_data = temp.path().join("app-data");
        let october = Self {
            store: PlacementStore::in_app_data(&app_data),
            settings: SettingsStore::in_app_data(&app_data),
            temp,
        };
        let root = october.root();
        let moves: Vec<_> = OCTOBER_FILES
            .iter()
            .map(|&(place, relative, date)| {
                let trip = placement(SuggestionKind::Trip, 2025, USA, place);
                moved_into(&root, &trip, relative, date)
            })
            .collect();
        place_on_disk(&october.store, &root, &moves);
        october
    }

    fn selected(&self) -> PathBuf {
        self.temp.path().join("todo")
    }

    pub(super) fn root(&self) -> PathBuf {
        self.selected().join("Processed")
    }

    /// The October folder of `place`: `Processed/2025/United States/<place>`.
    pub(super) fn placed(&self, place: &str) -> PathBuf {
        self.root().join("2025").join(USA).join(place)
    }

    /// The November scan's trip, continued as detection does it.
    pub(super) fn november_trip(&self) -> PlaceSuggestion {
        let vancouver = place_group(
            "Vancouver",
            "2025-11-01 09:00:00",
            "2025-11-01 11:00:00",
            &["v1"],
        );
        let trip = detected(
            SuggestionKind::Trip,
            "United States & Canada",
            vec![vancouver],
        );
        let selected = self.selected();
        continue_placed_suggestions(&self.store, &self.settings, &selected, &[], &[], vec![trip])
            .expect("continued")
            .remove(0)
    }

    /// The November trip moved to January 2, 2026, with Vancouver renamed to `place`.
    pub(super) fn january_trip(&self, place: &str) -> PlaceSuggestion {
        let mut trip = self.november_trip();
        let continues = trip.continues.as_mut().expect("continues");
        continues.end = "2025-12-31 20:00:00".to_owned();
        let january = "2026-01-02 09:00:00";
        trip.places[0] = place_group(place, january, january, &["v1"]);
        (trip.start, trip.end) = (january.to_owned(), january.to_owned());
        trip
    }

    pub(super) fn placed_moves(&self, trip: &PlaceSuggestion) -> Vec<PlacedMove> {
        let metadata = [dated_metadata("v1", "2025-11-01 09:00:00")];
        let selected = self.selected();
        fs::create_dir_all(&selected).expect("selected folder");
        plan_continued_moves(
            &self.settings,
            &self.store,
            &selected,
            std::slice::from_ref(trip),
            &metadata,
        )
        .expect("planned")
    }
}
