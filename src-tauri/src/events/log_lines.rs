//! Readable log lines for events whose payload needs more than one field.

use crate::models::{HistoryReplayProgressEvent, MoveResult};

pub(super) fn log_move_result(result: &MoveResult) {
    let level = match result.error {
        Some(_) => log::Level::Warn,
        None => log::Level::Info,
    };
    log::log!(
        level,
        "{:?} {} -> {}{}",
        result.result_status,
        result.attempted_source_path,
        result.attempted_destination_path,
        result
            .error
            .as_deref()
            .map(|error| format!(": {error}"))
            .unwrap_or_default()
    );
}

pub(super) fn log_replay_step(direction: &str, payload: &HistoryReplayProgressEvent) {
    let operation = &payload.operation;
    log::info!(
        "{direction} {}/{} {:?} {} -> {}",
        payload.completed_operations,
        payload.total_operations,
        operation.status,
        operation.source_path,
        operation.destination_path
    );
}
