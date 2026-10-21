use crate::{
    history,
    models::{HistoryOperation, HistoryRunDetails, HistoryRunStatus},
};

pub(super) fn undoable_operations(
    details: &HistoryRunDetails,
) -> Result<Vec<HistoryOperation>, String> {
    ensure_safe_for_replay(details)?;
    if !matches!(
        details.run.run_status,
        HistoryRunStatus::Completed
            | HistoryRunStatus::CompletedWithErrors
            | HistoryRunStatus::Redone
    ) {
        return Err("Undo is only available for completed or redone runs.".to_string());
    }

    let operations = history::collect_undoable_operations(&details.operations);
    if operations.is_empty() {
        return Err("No undoable operations were found for this history run.".to_string());
    }
    Ok(operations)
}

pub(super) fn redoable_operations(
    details: &HistoryRunDetails,
) -> Result<Vec<HistoryOperation>, String> {
    ensure_safe_for_replay(details)?;
    if !matches!(
        details.run.run_status,
        HistoryRunStatus::Undone | HistoryRunStatus::PartiallyUndone
    ) {
        return Err("Redo is only available for undone runs.".to_string());
    }

    let operations = history::collect_redoable_operations(&details.operations);
    if operations.is_empty() {
        return Err("No redoable operations were found for this history run.".to_string());
    }
    Ok(operations)
}

fn ensure_safe_for_replay(details: &HistoryRunDetails) -> Result<(), String> {
    if details.unsafe_for_undo_redo {
        return Err("History run is marked unsafe for undo/redo.".to_string());
    }
    Ok(())
}
