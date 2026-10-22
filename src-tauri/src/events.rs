//! Backend event names and typed emit helpers.
//!
//! Domain modules emit through [`EventSink`]; only this file knows the Tauri emitter.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

mod log_lines;

use log_lines::{log_move_result, log_replay_step};

use crate::models::{
    HistoryIndexUpdatedEvent, HistoryReplayProgressEvent, MetadataFinishedEvent,
    MetadataResultsEvent, MoveFileEvent, MoveFinishedEvent, MoveProgressEvent, PlanErrorEvent,
    PlanFileEvent, PlanFinishedEvent, ScanErrorEvent, ScanFilesEvent, ScanFinishedEvent,
    ScanStartedEvent,
};

pub const SCAN_STARTED: &str = "scan:started";
pub const SCAN_FILES: &str = "scan:files";
pub const SCAN_ERROR: &str = "scan:error";
pub const SCAN_FINISHED: &str = "scan:finished";
pub const METADATA_RESULTS: &str = "metadata:results";
pub const METADATA_FINISHED: &str = "metadata:finished";
pub const PLAN_FILE: &str = "plan:file";
pub const PLAN_ERROR: &str = "plan:error";
pub const PLAN_FINISHED: &str = "plan:finished";
pub const MOVE_FILE: &str = "move:file";
pub const MOVE_PROGRESS: &str = "move:progress";
pub const MOVE_FINISHED: &str = "move:finished";
pub const HISTORY_INDEX_UPDATED: &str = "history:index-updated";
pub const HISTORY_UNDO_PROGRESS: &str = "history:undo-progress";
pub const HISTORY_REDO_PROGRESS: &str = "history:redo-progress";

/// Receives backend events; the app forwards them to the webview, tests record them.
pub(crate) trait EventSink {
    fn emit<P: Serialize + Clone>(&self, event: &'static str, payload: P);
}

impl<R: Runtime> EventSink for AppHandle<R> {
    fn emit<P: Serialize + Clone>(&self, event: &'static str, payload: P) {
        // Emitting only fails once the webview is gone, when no listener is left to tell.
        let _ = Emitter::emit(self, event, payload);
    }
}

pub(crate) fn scan_started(sink: &impl EventSink, payload: ScanStartedEvent) {
    log::info!("Scan started in {}", payload.root);
    sink.emit(SCAN_STARTED, payload);
}

pub(crate) fn scan_files(sink: &impl EventSink, payload: ScanFilesEvent) {
    for record in &payload.records {
        log::debug!("Scanned {} ({:?})", record.source_path, record.scan_status);
    }
    sink.emit(SCAN_FILES, payload);
}

pub(crate) fn scan_error(sink: &impl EventSink, payload: ScanErrorEvent) {
    log::warn!("Scan error at {}: {}", payload.path, payload.error);
    sink.emit(SCAN_ERROR, payload);
}

pub(crate) fn scan_finished(sink: &impl EventSink, payload: ScanFinishedEvent) {
    log::info!(
        "Scan finished: {} found, {} skipped, {} errors, cancelled: {}",
        payload.discovered,
        payload.skipped,
        payload.errors,
        payload.cancelled
    );
    sink.emit(SCAN_FINISHED, payload);
}

pub(crate) fn metadata_results(sink: &impl EventSink, payload: MetadataResultsEvent) {
    for result in &payload.results {
        log::debug!("Read {} ({:?})", result.file_id, result.metadata_status);
    }
    sink.emit(METADATA_RESULTS, payload);
}

pub(crate) fn metadata_finished(sink: &impl EventSink, payload: MetadataFinishedEvent) {
    log::info!(
        "Metadata finished for scan {}, cancelled: {}",
        payload.scan_id,
        payload.cancelled
    );
    sink.emit(METADATA_FINISHED, payload);
}

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

pub(crate) fn move_file(sink: &impl EventSink, payload: MoveFileEvent) {
    log_move_result(&payload.result);
    sink.emit(MOVE_FILE, payload);
}

pub(crate) fn move_progress(sink: &impl EventSink, payload: MoveProgressEvent) {
    log::debug!("Move {:?}: {}", payload.phase, payload.message);
    sink.emit(MOVE_PROGRESS, payload);
}

pub(crate) fn move_finished(sink: &impl EventSink, payload: MoveFinishedEvent) {
    log::info!(
        "Move finished: {} moved, {} copied, {} skipped, {} failed",
        payload.moved,
        payload.copied,
        payload.skipped,
        payload.failed
    );
    sink.emit(MOVE_FINISHED, payload);
}

pub(crate) fn history_index_updated(sink: &impl EventSink, run_id: &str) {
    log::debug!("History index updated for run {run_id}");
    let run_id = Some(run_id.to_string());
    sink.emit(HISTORY_INDEX_UPDATED, HistoryIndexUpdatedEvent { run_id });
}

pub(crate) fn history_undo_progress(sink: &impl EventSink, payload: HistoryReplayProgressEvent) {
    log_replay_step("Undo", &payload);
    sink.emit(HISTORY_UNDO_PROGRESS, payload);
}

pub(crate) fn history_redo_progress(sink: &impl EventSink, payload: HistoryReplayProgressEvent) {
    log_replay_step("Redo", &payload);
    sink.emit(HISTORY_REDO_PROGRESS, payload);
}
