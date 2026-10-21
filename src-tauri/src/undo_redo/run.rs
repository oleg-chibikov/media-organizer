use tauri::AppHandle;

use super::{
    records::{build_redo_history_operation, build_undo_history_operation},
    redo::execute_redo_operation,
    replay::{completion_status, finish_replay, replay_operations},
    undo::execute_undo_operation,
};
use crate::{
    events,
    models::{
        HistoryOperation, HistoryRedoProgressEvent, HistoryRunStatus, HistoryUndoProgressEvent,
    },
};

pub(crate) fn run_undo(
    app: &AppHandle,
    run_id: &str,
    initial_status: HistoryRunStatus,
    operations: &[HistoryOperation],
) {
    let summary = replay_operations(
        app,
        run_id,
        operations,
        execute_undo_operation,
        build_undo_history_operation,
        |total_operations, completed_operations, operation| {
            let _ = events::emit_history_undo_progress(
                app,
                HistoryUndoProgressEvent {
                    run_id: run_id.to_string(),
                    total_operations,
                    completed_operations,
                    operation,
                },
            );
        },
    );
    let final_status = completion_status(&initial_status, &summary, HistoryRunStatus::Undone);
    finish_replay(app, run_id, &initial_status, final_status);
}

pub(crate) fn run_redo(
    app: &AppHandle,
    run_id: &str,
    initial_status: HistoryRunStatus,
    operations: &[HistoryOperation],
) {
    let summary = replay_operations(
        app,
        run_id,
        operations,
        execute_redo_operation,
        build_redo_history_operation,
        |total_operations, completed_operations, operation| {
            let _ = events::emit_history_redo_progress(
                app,
                HistoryRedoProgressEvent {
                    run_id: run_id.to_string(),
                    total_operations,
                    completed_operations,
                    operation,
                },
            );
        },
    );
    let final_status = completion_status(&initial_status, &summary, HistoryRunStatus::Redone);
    finish_replay(app, run_id, &initial_status, final_status);
}
