//! The scan's plans. The backend plans each file once its date is read and the person's
//! place folders are known, so the webview only mirrors the `plan:*` events.

mod placed;
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
    models::{
        MetadataResult, MovePlan, PlaceSuggestion, PlacedFilesEvent, PlanClearedEvent,
        PlanSettledEvent, PlanStatus,
    },
    placements::PlacementStore,
    planner::Reservations,
    settings::SettingsStore,
};

/// What the session plans with, resolved when the scan starts.
#[derive(Clone)]
pub(crate) struct PlanContext {
    pub folder: PathBuf,
    pub settings: SettingsStore,
    pub placements: PlacementStore,
}

#[derive(Default)]
pub(super) struct Planning {
    /// The place folders the person chose. `None` holds plans back until they choose.
    places: Option<Vec<PlaceSuggestion>>,
    place_keys: HashMap<String, String>,
    signatures: HashMap<String, String>,
    plans: HashMap<String, MovePlan>,
    reservations: Reservations,
    /// Files earlier runs placed that move with a continued folder. Date planning skips them.
    placed_ids: HashSet<String>,
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

    /// Plans every file with a date for the place folders the person chose, and moves the
    /// placed files those folders continue. Fails only when the placed files can't be planned.
    pub(crate) fn set_places(
        &self,
        sink: &impl EventSink,
        suggestions: Vec<PlaceSuggestion>,
    ) -> Result<(), String> {
        let mut planning = self.planning();
        planning.place_keys = signature::place_keys(&suggestions);
        planning.places = Some(suggestions);
        let placed = self.plan_placed(sink, &mut planning);
        let file_ids = self.read_file_ids();
        self.plan_files(sink, &mut planning, &file_ids);
        let scan_id = self.scan_id;
        events::plan_settled(sink, PlanSettledEvent { scan_id });
        self.moves_changed.notify_all();
        placed
    }

    /// Drops every plan, as for a new destination, and waits for the place folders again.
    pub(crate) fn clear_plans(&self, sink: &impl EventSink) {
        let mut planning = self.planning();
        let dropped: Vec<String> = planning.placed_ids.drain().collect();
        self.forget(&dropped);
        *planning = Planning::default();
        let scan_id = self.scan_id;
        if !dropped.is_empty() {
            let added = Vec::new();
            events::plan_placed(
                sink,
                PlacedFilesEvent {
                    scan_id,
                    added,
                    dropped,
                },
            );
        }
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

    pub(super) fn plan_ids_with(&self, keep: impl Fn(&PlanStatus) -> bool) -> Vec<String> {
        let planning = self.planning();
        planning
            .plans
            .values()
            .filter(|plan| keep(&plan.plan_status))
            .map(|plan| plan.file_id.clone())
            .collect()
    }

    fn planning(&self) -> MutexGuard<'_, Planning> {
        self.planning.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
