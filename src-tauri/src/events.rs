use tauri::{AppHandle, Emitter};

use crate::models::{ScanErrorEvent, ScanFileEvent, ScanFinishedEvent, ScanStartedEvent};

pub const SCAN_STARTED: &str = "scan:started";
pub const SCAN_FILE: &str = "scan:file";
pub const SCAN_SKIPPED: &str = "scan:skipped";
pub const SCAN_ERROR: &str = "scan:error";
pub const SCAN_FINISHED: &str = "scan:finished";

pub fn emit_scan_started(app: &AppHandle, payload: ScanStartedEvent) -> Result<(), String> {
    app.emit(SCAN_STARTED, payload).map_err(|e| e.to_string())
}

pub fn emit_scan_file(app: &AppHandle, payload: ScanFileEvent) -> Result<(), String> {
    app.emit(SCAN_FILE, payload).map_err(|e| e.to_string())
}

pub fn emit_scan_skipped(app: &AppHandle, payload: ScanFileEvent) -> Result<(), String> {
    app.emit(SCAN_SKIPPED, payload).map_err(|e| e.to_string())
}

pub fn emit_scan_error(app: &AppHandle, payload: ScanErrorEvent) -> Result<(), String> {
    app.emit(SCAN_ERROR, payload).map_err(|e| e.to_string())
}

pub fn emit_scan_finished(app: &AppHandle, payload: ScanFinishedEvent) -> Result<(), String> {
    app.emit(SCAN_FINISHED, payload).map_err(|e| e.to_string())
}
