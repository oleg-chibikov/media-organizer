use std::path::Path;

use tauri::AppHandle;

use super::{
    index::{load_history_index, write_history_index},
    index_updates::keep_newest_runs,
    paths::history_root_for_app,
    recovery::{recover_history_run_from_entry, RecoveredHistoryRun},
};
use crate::models::{
    HistoryIndexEntry, HistoryOperationKind, HistoryOperationStatus, HistoryRunDetails,
};

pub(crate) fn list_history_runs(app: &AppHandle) -> Result<Vec<HistoryIndexEntry>, String> {
    list_history_runs_from_root(&history_root_for_app(app)?)
}

pub(crate) fn get_history_run(
    app: &AppHandle,
    run_id: &str,
) -> Result<Option<HistoryRunDetails>, String> {
    get_history_run_from_root(&history_root_for_app(app)?, run_id)
}

/// Lists runs with state recovered from their logs, persisting any index drift.
pub(super) fn list_history_runs_from_root(
    history_root: &Path,
) -> Result<Vec<HistoryIndexEntry>, String> {
    let mut index = load_history_index(history_root)?;
    let mut recovered_entries: Vec<HistoryIndexEntry> =
        index.runs.iter().map(refreshed_index_entry).collect();
    keep_newest_runs(&mut recovered_entries);

    if recovered_entries != index.runs {
        index.runs = recovered_entries.clone();
        write_history_index(history_root, &index)?;
    }

    Ok(recovered_entries)
}

pub(super) fn get_history_run_from_root(
    history_root: &Path,
    run_id: &str,
) -> Result<Option<HistoryRunDetails>, String> {
    let index = load_history_index(history_root)?;
    let Some(entry) = index.runs.iter().find(|entry| entry.run_id == run_id) else {
        return Ok(None);
    };

    let recovered = recover_history_run_from_entry(entry);
    Ok(Some(HistoryRunDetails {
        run: recovered.run,
        operations: recovered
            .operations
            .into_iter()
            .filter(|op| matches!(op.operation_kind, HistoryOperationKind::Process))
            .collect(),
        unsafe_for_undo_redo: recovered.unsafe_for_undo_redo,
        recovery_error: recovered.recovery_error,
    }))
}

fn refreshed_index_entry(entry: &HistoryIndexEntry) -> HistoryIndexEntry {
    let recovered = recover_history_run_from_entry(entry);
    let mut next_entry = entry.clone();
    next_entry.destination_root_path = recovered.run.destination_root_path.clone();
    next_entry.completed_at = recovered.run.completed_at.clone();
    next_entry.run_status = recovered.run.run_status.clone();
    next_entry.files_migrated = files_migrated(&recovered);
    next_entry.undo_available = recovered.undo_available();
    next_entry.redo_available = recovered.redo_available();
    next_entry
}

fn files_migrated(recovered: &RecoveredHistoryRun) -> u64 {
    recovered
        .operations
        .iter()
        .filter(|operation| {
            matches!(operation.operation_kind, HistoryOperationKind::Process)
                && matches!(operation.status, HistoryOperationStatus::Succeeded)
        })
        .count() as u64
}
