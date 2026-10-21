use std::path::{Path, PathBuf};

use super::{
    progress::emit_move_progress_message,
    recorder::MoveRunRecorder,
    request::{MoveRunRequest, RunLookups},
    source_cleanup,
};
use crate::{
    events::EventSink,
    history::ActiveHistoryRun,
    models::{MoveProgressPhase, MoveResultStatus},
    mover::{self, MoveOperation},
};

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
        let safety_checked = mover::validate_move_safety_with_roots(
            &request.selected_folder,
            &request.destination_root,
            vec![operation],
        );
        recorder.record_all(safety_checked.skipped);

        let rechecked = mover::recheck_destination_conflicts(safety_checked.ready);
        recorder.record_all(rechecked.skipped);

        for operation in rechecked.ready {
            progress(
                MoveProgressPhase::Moving,
                Some(&operation.file_id),
                "Moving file.",
                recorder.completed_operations(),
            );
            moved_source_directories.extend(move_file(&operation, &mut recorder));
        }
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

/// Moves one file and returns its source folder when the file left it.
fn move_file(
    operation: &MoveOperation,
    recorder: &mut MoveRunRecorder<impl EventSink>,
) -> Option<PathBuf> {
    let source_parent = Path::new(&operation.source_path)
        .parent()
        .map(Path::to_path_buf);
    let result = mover::execute_move_with_rename(operation);
    let moved = matches!(result.result_status, MoveResultStatus::Moved);
    recorder.record(result);
    source_parent.filter(|_| moved)
}
