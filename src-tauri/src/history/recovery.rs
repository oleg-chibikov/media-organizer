use std::path::PathBuf;

use super::{
    candidates::{collect_redoable_operations, collect_undoable_operations},
    log_parser::parse_run_log_records,
    HISTORY_VERSION,
};
use crate::models::{HistoryIndexEntry, HistoryOperation, HistoryRun, HistoryRunStatus};

/// A run rebuilt from its log; damaged logs are flagged unsafe for undo/redo.
#[derive(Debug)]
pub(super) struct RecoveredHistoryRun {
    pub run: HistoryRun,
    pub operations: Vec<HistoryOperation>,
    pub unsafe_for_undo_redo: bool,
    pub recovery_error: Option<String>,
}

impl RecoveredHistoryRun {
    pub(super) fn undo_available(&self) -> bool {
        !self.unsafe_for_undo_redo
            && matches!(
                self.run.run_status,
                HistoryRunStatus::Completed
                    | HistoryRunStatus::CompletedWithErrors
                    | HistoryRunStatus::Redone
            )
            && !collect_undoable_operations(&self.operations).is_empty()
    }

    pub(super) fn redo_available(&self) -> bool {
        !self.unsafe_for_undo_redo
            && matches!(
                self.run.run_status,
                HistoryRunStatus::Undone | HistoryRunStatus::PartiallyUndone
            )
            && !collect_redoable_operations(&self.operations).is_empty()
    }
}

pub(super) fn recover_history_run_from_entry(entry: &HistoryIndexEntry) -> RecoveredHistoryRun {
    let history_path = PathBuf::from(&entry.history_file_path);
    let parse_result = parse_run_log_records(&history_path, &entry.run_id);
    let mut run = parse_result
        .started_run
        .unwrap_or_else(|| run_from_index_entry(entry));
    run.history_file_path = entry.history_file_path.clone();

    let mut unsafe_for_undo_redo = false;
    if parse_result.corrupt {
        run.run_status = HistoryRunStatus::Corrupt;
        unsafe_for_undo_redo = true;
    } else if parse_result.partial || parse_result.incomplete {
        run.run_status = HistoryRunStatus::Partial;
        unsafe_for_undo_redo = true;
    } else if let Some(completed) = parse_result.completed_record {
        run.completed_at = Some(completed.completed_at);
        run.run_status = completed.run_status;
    }

    if parse_result.recovery_error.is_some() {
        unsafe_for_undo_redo = true;
    }

    RecoveredHistoryRun {
        run,
        operations: parse_result.operations,
        unsafe_for_undo_redo,
        recovery_error: parse_result.recovery_error,
    }
}

fn run_from_index_entry(entry: &HistoryIndexEntry) -> HistoryRun {
    HistoryRun {
        run_id: entry.run_id.clone(),
        history_version: HISTORY_VERSION,
        app_version: String::new(),
        selected_folder: entry.selected_folder.clone(),
        folder_label: entry.folder_label.clone(),
        destination_root_mode: None,
        destination_root_path: entry.destination_root_path.clone(),
        started_at: entry.started_at.clone(),
        completed_at: entry.completed_at.clone(),
        run_status: entry.run_status.clone(),
        history_file_path: entry.history_file_path.clone(),
    }
}
