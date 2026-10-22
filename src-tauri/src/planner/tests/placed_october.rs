use std::{
    fs,
    path::{Path, PathBuf},
};

use tempfile::{tempdir, TempDir};

use super::{dated, photo_record};
use crate::{
    events,
    models::{PlaceSuggestion, PlacedMove, SuggestionKind},
    placements::{continue_placed_suggestions, PlacementStore},
    planner::{plan_continued_moves, stream_move_plan, PlaceFolders},
    settings::SettingsStore,
    test_support::{
        events::RecordingSink,
        media::dated_metadata,
        placements::{detected, moved_into, place_group, place_on_disk, placement},
    },
};

pub(super) const USA: &str = "United States";

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
        let (store, settings) = (
            PlacementStore::in_app_data(&app_data),
            SettingsStore::in_app_data(&app_data),
        );
        let root = temp.path().join("todo").join("Processed");
        let usa = |place| placement(SuggestionKind::Trip, 2025, USA, place);
        place_on_disk(
            &store,
            &root,
            &[
                moved_into(
                    &root,
                    &usa(Some("New York")),
                    "2025/United States/New York/10-28/a.jpg",
                    "2025-10-28 10:00:00",
                ),
                moved_into(
                    &root,
                    &usa(Some("New York")),
                    "2025/United States/New York/10-30/b.jpg",
                    "2025-10-30 10:00:00",
                ),
                moved_into(
                    &root,
                    &usa(None),
                    "2025/United States/10-30/r.jpg",
                    "2025-10-30 20:00:00",
                ),
                moved_into(
                    &root,
                    &usa(Some("Vancouver")),
                    "2025/United States/Vancouver/c.jpg",
                    "2025-10-31 18:00:00",
                ),
            ],
        );
        Self {
            temp,
            store,
            settings,
        }
    }

    pub(super) fn root(&self) -> PathBuf {
        self.temp.path().join("todo").join("Processed")
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
        let selected = self.temp.path().join("todo");
        continue_placed_suggestions(&self.store, &self.settings, &selected, &[], &[], vec![trip])
            .expect("continued")
            .remove(0)
    }

    pub(super) fn placed_moves(&self, trip: &PlaceSuggestion) -> Vec<PlacedMove> {
        let metadata = [dated_metadata("v1", "2025-11-01 09:00:00")];
        let selected = self.temp.path().join("todo");
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

pub(super) fn planned_dir(root: &Path, trip: &PlaceSuggestion, id: &str, date: &str) -> PathBuf {
    let layout = dated(root);
    let metadata = [dated_metadata(id, date)];
    let folders = PlaceFolders::new(std::slice::from_ref(trip), &metadata).expect("folders");
    let sink = RecordingSink::default();
    stream_move_plan(
        &sink,
        0,
        &layout,
        &[photo_record(id, "jpg")],
        &metadata,
        &folders,
    );
    PathBuf::from(
        sink.payloads(events::PLAN_FILE)[0]["plan"]["destination_dir"]
            .as_str()
            .expect("a dir"),
    )
}

/// The November trip moved to January 2, 2026, with Vancouver renamed to `place`.
pub(super) fn january_trip(october: &PlacedOctober, place: &str) -> PlaceSuggestion {
    let mut trip = october.november_trip();
    let continues = trip.continues.as_mut().expect("continues");
    continues.end = "2025-12-31 20:00:00".to_owned();
    let january = "2026-01-02 09:00:00";
    trip.places[0] = place_group(place, january, january, &["v1"]);
    (trip.start, trip.end) = (january.to_owned(), january.to_owned());
    trip
}
