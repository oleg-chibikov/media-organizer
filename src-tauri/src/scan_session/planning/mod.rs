//! The scan's plans. The backend plans each file once its date is read and the person's
//! place folders are known, so the webview only mirrors the `plan:*` events.

mod library;
mod plan_files;
mod signature;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{MutexGuard, PoisonError},
};

use super::ScanSession;
use crate::{
    events::{self, EventSink},
    models::{FilePlace, MetadataResult, MovePlan, PlanClearedEvent, PlanSettledEvent, PlanStatus},
    planner::{DestinationLayout, LibraryMoves, PlaceFolders, Reservations},
    settings::SettingsStore,
};

/// What the session plans with, resolved when the scan starts.
#[derive(Clone)]
pub(crate) struct PlanContext {
    pub folder: PathBuf,
    pub settings: SettingsStore,
}

#[derive(Default)]
pub(super) struct Planning {
    /// The place folder of each file. `None` holds plans back until places are known.
    places: Option<PlaceFolders>,
    /// The places came from a preview while dates are still read, so no file moves yet.
    preview: bool,
    /// Where the files go, resolved on the first plan after the places are set.
    layout: Option<DestinationLayout>,
    signatures: HashMap<String, String>,
    plans: HashMap<String, MovePlan>,
    reservations: Reservations,
    /// The places last applied, so a new choice about earlier days can plan again.
    file_places: Option<HashMap<String, FilePlace>>,
    /// The person chose to leave earlier days of the scan's places where they are.
    library_apart: bool,
    /// Files of earlier runs that move into their place's new folder.
    library: LibraryMoves,
}

impl ScanSession {
    /// Plans files whose date was just read, once the place folders are known.
    pub(crate) fn plan_read(&self, sink: &impl EventSink, results: &[MetadataResult]) {
        let mut planning = self.planning();
        // A preview misses the place of a file read after it, so the file waits for the next.
        if planning.places.is_some() && !planning.preview {
            let file_ids: Vec<String> = results.iter().map(|r| r.file_id.clone()).collect();
            self.plan_files(sink, &mut planning, &file_ids);
        }
    }

    /// Plans every file with a date for its place folder, or a date folder when `file_places`
    /// has none for it. Plans of a `preview` show where files will likely go, and don't move.
    pub(crate) fn set_places(
        &self,
        sink: &impl EventSink,
        file_places: HashMap<String, FilePlace>,
        preview: bool,
    ) {
        let mut planning = self.planning();
        planning.preview = preview;
        let file_ids = self.read_file_ids();
        let folders = PlaceFolders::new(&file_places, &self.all_metadata());
        planning.file_places = Some(file_places);
        match folders {
            Ok(folders) => {
                planning.places = Some(folders);
                planning.layout = None;
                self.fit_destination(&mut planning);
                self.plan_files(sink, &mut planning, &file_ids);
            }
            Err(error) => {
                self.drop_library(&mut planning);
                planning.places = Some(PlaceFolders::default());
                planning.layout = None;
                self.fail_all(sink, &file_ids, &error);
            }
        }
        let scan_id = self.scan_id;
        events::plan_settled(sink, PlanSettledEvent { scan_id });
        self.moves_changed.notify_all();
    }

    /// Drops every plan, as for a new destination, and waits for the place folders again.
    pub(crate) fn clear_plans(&self, sink: &impl EventSink) {
        let mut planning = self.planning();
        self.drop_library(&mut planning);
        let library_apart = planning.library_apart;
        *planning = Planning {
            library_apart,
            ..Planning::default()
        };
        drop(planning);
        let scan_id = self.scan_id;
        events::plan_cleared(sink, PlanClearedEvent { scan_id });
    }

    /// The current plans of `file_ids` in their order. Fails on a file with no plan.
    pub(crate) fn plans(&self, file_ids: &[String]) -> Result<Vec<MovePlan>, String> {
        let planning = self.planning();
        file_ids
            .iter()
            .map(|file_id| {
                planning
                    .plans
                    .get(file_id)
                    .cloned()
                    .ok_or_else(|| format!("File {file_id} has no plan."))
            })
            .collect()
    }

    pub(super) fn waiting_for_places(&self) -> bool {
        let planning = self.planning();
        planning.places.is_none() || planning.preview
    }

    /// The files whose plan is ready and not in `done`, in a stable order.
    pub(super) fn ready_file_ids(&self, done: &HashSet<String>) -> Vec<String> {
        let planning = self.planning();
        if planning.preview {
            return Vec::new();
        }
        let mut ids: Vec<String> = planning
            .plans
            .values()
            .filter(|plan| matches!(plan.plan_status, PlanStatus::Ready))
            .filter(|plan| !done.contains(&plan.file_id))
            .map(|plan| plan.file_id.clone())
            .collect();
        ids.sort_unstable();
        ids
    }

    fn planning(&self) -> MutexGuard<'_, Planning> {
        self.planning.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
