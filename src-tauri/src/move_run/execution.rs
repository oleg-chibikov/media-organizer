use super::{
    emptied_place_folders::remove_emptied_place_folders,
    file_step::{move_file, recheck_operation},
    placement_record::record_placements,
    progress::emit_move_progress_message,
    recorder::MoveRunRecorder,
    request::{MoveRunRequest, RunLookups},
};
use tokio_util::sync::CancellationToken;

use crate::{events::EventSink, history::ActiveHistoryRun, models::MoveProgressPhase, mover};

const UNLOGGED_MOVE_REASON: &str =
    "Not moved: the history log stopped recording, so this move couldn't be undone.";

/// Checks, moves and records every operation of one batch, until `pause` is cancelled.
/// Files the run didn't reach get no result and stay at the source.
pub(super) fn run_move_plan(
    sink: &impl EventSink,
    request: MoveRunRequest,
    history_run: ActiveHistoryRun,
    pause: &CancellationToken,
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

    recorder.record_all(prepared.skipped);

    for operation in prepared.operations {
        if pause.is_cancelled() {
            break;
        }
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
        move_file(&operation, &mut recorder);
    }

    progress(
        MoveProgressPhase::Finalizing,
        None,
        "Finalizing history.",
        recorder.completed_operations(),
    );
    record_placements(&request, recorder.results());
    remove_emptied_place_folders(&request, recorder.results());
    recorder.finish();
}
