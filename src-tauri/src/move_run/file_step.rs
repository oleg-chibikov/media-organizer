use super::{recorder::MoveRunRecorder, request::MoveRunRequest};
use crate::{
    events::EventSink,
    mover::{self, MoveOperation},
};

/// Runs the root and conflict checks right before the move; a failed check is recorded.
pub(super) fn recheck_operation(
    request: &MoveRunRequest,
    operation: MoveOperation,
    recorder: &mut MoveRunRecorder<impl EventSink>,
) -> Option<MoveOperation> {
    let safety_checked = mover::validate_move_safety_with_roots(
        &request.selected_folder,
        &request.destination_root,
        vec![operation],
    );
    recorder.record_all(safety_checked.skipped);
    let rechecked = mover::recheck_destination_conflicts(safety_checked.ready);
    recorder.record_all(rechecked.skipped);
    rechecked.ready.into_iter().next()
}

/// Moves one file and records the result.
pub(super) fn move_file(operation: &MoveOperation, recorder: &mut MoveRunRecorder<impl EventSink>) {
    recorder.remember_source_size(&operation.file_id);
    recorder.record(mover::execute_move_with_rename(operation));
}
