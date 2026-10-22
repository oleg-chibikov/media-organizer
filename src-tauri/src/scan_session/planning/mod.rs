//! The scan's plans. The backend plans each file once its date is read and the person's
//! place folders are known, so the webview only mirrors the `plan:*` events.

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
    planner::{DestinationLayout, PlaceFolders, Reservations},
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
    /// Where the files go, resolved on the first plan after the places are set.
    layout: Option<DestinationLayout>,
    signatures: HashMap<String, String>,
    plans: HashMap<String, MovePlan>,
    reservations: Reservations,
}

impl ScanSession {
    /// Plans files whose date was just read, once the place folders are known.
    pub(crate) fn plan_read(&self, sink: &impl EventSink, results: &[MetadataResult]) {
        let mut planning = self.planning();
        if planning.places.is_some() {
            let file_ids: Vec<String> = results.iter().map(|r| r.file_id.clone()).collect();
            self.plan_files(sink, &mut planning, &file_ids);
        }
    }

    /// Plans every file with a date for its place folder, or a date folder when `file_places`
    /// has none for it.
    pub(crate) fn set_places(
        &self,
        sink: &impl EventSink,
        file_places: HashMap<String, FilePlace>,
    ) {
        let mut planning = self.planning();
        let file_ids = self.read_file_ids();
        match PlaceFolders::new(&file_places, &self.all_metadata()) {
            Ok(folders) => {
                planning.places = Some(folders);
                planning.layout = None;
                self.plan_files(sink, &mut planning, &file_ids);
            }
            Err(error) => {
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
        *self.planning() = Planning::default();
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
        self.planning().places.is_none()
    }

    /// The files whose plan is ready and not in `done`, in a stable order.
    pub(super) fn ready_file_ids(&self, done: &HashSet<String>) -> Vec<String> {
        let planning = self.planning();
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
