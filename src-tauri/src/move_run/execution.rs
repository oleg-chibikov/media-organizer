use super::{
    file_step::{move_file, recheck_operation},
    progress::emit_move_progress_message,
    recorder::MoveRunRecorder,
    request::{MoveRunRequest, RunLookups},
    source_cleanup,
};
use crate::{events::EventSink, history::ActiveHistoryRun, models::MoveProgressPhase, mover};

const UNLOGGED_MOVE_REASON: &str =
    "Not moved: the history log stopped recording, so this move couldn't be undone.";

/// Checks, moves and records every operation of one batch, then cleans emptied source folders.
pub(super) fn run_move_plan(
    sink: &impl EventSink,
    request: MoveRunRequest,
    history_run: ActiveHistoryRun,
) {
    let batch = &request.batch;
    let total = batch.plans.len() as u64;
    let progress = |phase, file_id: Option<&str>, message: &str, completed| {
        emit_move_progress_message(sink, phase, file_id, message, completed, total);
    };

    progress(
        MoveProgressPhase::Preparing,
        None,
        "Preparing move operations.",
        0,
    );
    let prepared = mover::prepare_move_operations(&batch.files, &batch.plans);
    let mut recorder = MoveRunRecorder::new(sink, history_run, RunLookups::new(batch));
    let mut moved_source_directories = Vec::new();

    recorder.record_all(prepared.skipped);

    for operation in prepared.operations {
        progress(
            MoveProgressPhase::Preparing,
            Some(&operation.file_id),
            "Checking source and destination.",
            recorder.completed_operations(),
        );
        if recorder.history_log_failed() {
            recorder.record(mover::skipped_operation(&operation, UNLOGGED_MOVE_REASON));
            continue;
        }
        let Some(operation) = recheck_operation(&request, operation, &mut recorder) else {
            continue;
        };
        progress(
            MoveProgressPhase::Moving,
            Some(&operation.file_id),
            "Moving file.",
            recorder.completed_operations(),
        );
        moved_source_directories.extend(move_file(&operation, &mut recorder));
    }

    progress(
        MoveProgressPhase::Finalizing,
        None,
        "Finalizing history and cleaning up source folders.",
        recorder.completed_operations(),
    );
    source_cleanup::cleanup_empty_source_directories(
        &request.selected_folder,
        &moved_source_directories,
    );
    recorder.finish();
}
