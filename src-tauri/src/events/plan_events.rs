use super::{
    EventSink, PLAN_CLEARED, PLAN_ERROR, PLAN_FILE, PLAN_FINISHED, PLAN_PLACED, PLAN_SETTLED,
};
use crate::models::{
    PlacedFilesEvent, PlanClearedEvent, PlanErrorEvent, PlanFileEvent, PlanFinishedEvent,
    PlanSettledEvent,
};

pub(crate) fn plan_file(sink: &impl EventSink, payload: PlanFileEvent) {
    log::debug!(
        "Planned {} -> {}",
        payload.plan.original_name,
        payload.plan.destination_path
    );
    sink.emit(PLAN_FILE, payload);
}

pub(crate) fn plan_error(sink: &impl EventSink, payload: PlanErrorEvent) {
    log::warn!("Plan error for {}: {}", payload.file_id, payload.error);
    sink.emit(PLAN_ERROR, payload);
}

pub(crate) fn plan_finished(sink: &impl EventSink, payload: PlanFinishedEvent) {
    log::info!(
        "Plan finished: {} planned, {} errors",
        payload.planned,
        payload.errors
    );
    sink.emit(PLAN_FINISHED, payload);
}

pub(crate) fn plan_cleared(sink: &impl EventSink, payload: PlanClearedEvent) {
    log::info!("Plans cleared for scan {}", payload.scan_id);
    sink.emit(PLAN_CLEARED, payload);
}

pub(crate) fn plan_placed(sink: &impl EventSink, payload: PlacedFilesEvent) {
    log::info!(
        "Placed files: {} join, {} leave",
        payload.added.len(),
        payload.dropped.len()
    );
    sink.emit(PLAN_PLACED, payload);
}

pub(crate) fn plan_settled(sink: &impl EventSink, payload: PlanSettledEvent) {
    log::info!("Plans settled for scan {}", payload.scan_id);
    sink.emit(PLAN_SETTLED, payload);
}
