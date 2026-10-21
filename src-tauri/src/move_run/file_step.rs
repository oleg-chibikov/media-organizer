use std::path::{Path, PathBuf};

use super::{recorder::MoveRunRecorder, request::MoveRunRequest};
use crate::{
    events::EventSink,
    models::MoveResultStatus,
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

/// Moves one file and returns its source folder when the file left it.
pub(super) fn move_file(
    operation: &MoveOperation,
    recorder: &mut MoveRunRecorder<impl EventSink>,
) -> Option<PathBuf> {
    let source_parent = Path::new(&operation.source_path)
        .parent()
        .map(Path::to_path_buf);
    recorder.remember_source_size(&operation.file_id);
    let result = mover::execute_move_with_rename(operation);
    let moved = matches!(result.result_status, MoveResultStatus::Moved);
    recorder.record(result);
    source_parent.filter(|_| moved)
}
