use super::{EventSink, PLAN_CLEARED, PLAN_FILES, PLAN_SETTLED};
use crate::models::{PlanClearedEvent, PlanFilesEvent, PlanSettledEvent};

pub(crate) fn plan_files(sink: &impl EventSink, payload: PlanFilesEvent) {
    for plan in &payload.plans {
        log::debug!(
            "Planned {} -> {}",
            plan.original_name,
            plan.destination_path
        );
    }
    for failed in &payload.errors {
        log::warn!("Plan error for {}: {}", failed.file_id, failed.error);
    }
    sink.emit(PLAN_FILES, payload);
}

pub(crate) fn plan_cleared(sink: &impl EventSink, payload: PlanClearedEvent) {
    log::info!("Plans cleared for scan {}", payload.scan_id);
    sink.emit(PLAN_CLEARED, payload);
}

pub(crate) fn plan_settled(sink: &impl EventSink, payload: PlanSettledEvent) {
    log::info!("Plans settled for scan {}", payload.scan_id);
    sink.emit(PLAN_SETTLED, payload);
}
