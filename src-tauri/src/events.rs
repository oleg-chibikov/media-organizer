//! Backend event names and typed emit helpers.
//!
//! Domain modules emit through [`EventSink`]; only this file knows the Tauri emitter.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

use crate::models::{
    HistoryIndexUpdatedEvent, HistoryReplayProgressEvent, MoveFileEvent, MoveFinishedEvent,
    MoveProgressEvent, PlanErrorEvent, PlanFileEvent, PlanFinishedEvent, ScanErrorEvent,
    ScanFileEvent, ScanFinishedEvent, ScanStartedEvent,
};

pub const SCAN_STARTED: &str = "scan:started";
pub const SCAN_FILE: &str = "scan:file";
pub const SCAN_SKIPPED: &str = "scan:skipped";
pub const SCAN_ERROR: &str = "scan:error";
pub const SCAN_FINISHED: &str = "scan:finished";
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
    sink.emit(SCAN_STARTED, payload);
}

pub(crate) fn scan_file(sink: &impl EventSink, payload: ScanFileEvent) {
    sink.emit(SCAN_FILE, payload);
}

pub(crate) fn scan_skipped(sink: &impl EventSink, payload: ScanFileEvent) {
    sink.emit(SCAN_SKIPPED, payload);
}

pub(crate) fn scan_error(sink: &impl EventSink, payload: ScanErrorEvent) {
    sink.emit(SCAN_ERROR, payload);
}

pub(crate) fn scan_finished(sink: &impl EventSink, payload: ScanFinishedEvent) {
    sink.emit(SCAN_FINISHED, payload);
}

pub(crate) fn plan_file(sink: &impl EventSink, payload: PlanFileEvent) {
    sink.emit(PLAN_FILE, payload);
}

pub(crate) fn plan_error(sink: &impl EventSink, payload: PlanErrorEvent) {
    sink.emit(PLAN_ERROR, payload);
}

pub(crate) fn plan_finished(sink: &impl EventSink, payload: PlanFinishedEvent) {
    sink.emit(PLAN_FINISHED, payload);
}

pub(crate) fn move_file(sink: &impl EventSink, payload: MoveFileEvent) {
    sink.emit(MOVE_FILE, payload);
}

pub(crate) fn move_progress(sink: &impl EventSink, payload: MoveProgressEvent) {
    sink.emit(MOVE_PROGRESS, payload);
}

pub(crate) fn move_finished(sink: &impl EventSink, payload: MoveFinishedEvent) {
    sink.emit(MOVE_FINISHED, payload);
}

pub(crate) fn history_index_updated(sink: &impl EventSink, run_id: &str) {
    let run_id = Some(run_id.to_string());
    sink.emit(HISTORY_INDEX_UPDATED, HistoryIndexUpdatedEvent { run_id });
}

pub(crate) fn history_undo_progress(sink: &impl EventSink, payload: HistoryReplayProgressEvent) {
    sink.emit(HISTORY_UNDO_PROGRESS, payload);
}

pub(crate) fn history_redo_progress(sink: &impl EventSink, payload: HistoryReplayProgressEvent) {
    sink.emit(HISTORY_REDO_PROGRESS, payload);
}
