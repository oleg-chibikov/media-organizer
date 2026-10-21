use std::{fs, io, path::Path};

use super::outcome::{ensure_expected_size, OperationOutcome};
use crate::{
    models::{HistoryOperation, MoveStrategy},
    safe_fs,
};

pub(super) fn execute_undo_operation(operation: &HistoryOperation) -> OperationOutcome {
    let result = match operation.move_strategy {
        MoveStrategy::Rename => undo_rename_operation(operation),
        MoveStrategy::CopyKeepSource => undo_copy_keep_source_operation(operation),
        MoveStrategy::NotAttempted => Err(OperationOutcome::blocked(
            "Operation strategy is not undoable.",
        )),
    };
    result.map_or_else(|outcome| outcome, |()| OperationOutcome::succeeded())
}

fn undo_rename_operation(operation: &HistoryOperation) -> Result<(), OperationOutcome> {
    let undo_source = Path::new(&operation.destination_path);
    let undo_destination = Path::new(&operation.source_path);

    if !safe_fs::is_plain_file(undo_source) {
        return Err(OperationOutcome::blocked(
            "Undo blocked: processed destination file is missing.",
        ));
    }
    if safe_fs::path_is_occupied(undo_destination) {
        return Err(OperationOutcome::blocked(
            "Undo blocked: original source path already exists.",
        ));
    }
    ensure_expected_size(
        undo_source,
        operation.expected_destination_size,
        "Undo",
        "destination",
    )?;

    if let Some(parent) = undo_destination.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            OperationOutcome::failed(format!(
                "Undo failed: could not create source parent directory: {error}"
            ))
        })?;
    }

    safe_fs::move_file_no_replace(undo_source, undo_destination).map_err(|error| {
        if error.kind() == io::ErrorKind::AlreadyExists {
            OperationOutcome::blocked("Undo blocked: original source path appeared during undo.")
        } else {
            OperationOutcome::failed(format!(
                "Undo failed: could not move file back to source: {error}"
            ))
        }
    })
}

fn undo_copy_keep_source_operation(operation: &HistoryOperation) -> Result<(), OperationOutcome> {
    let copied_destination = Path::new(&operation.destination_path);
    let original_source = Path::new(&operation.source_path);

    if !safe_fs::is_plain_file(original_source) {
        return Err(OperationOutcome::blocked(
            "Undo blocked: original source file is missing.",
        ));
    }
    if !safe_fs::is_plain_file(copied_destination) {
        return Err(OperationOutcome::blocked(
            "Undo blocked: copied destination file is missing.",
        ));
    }
    ensure_expected_size(
        original_source,
        operation.expected_source_size,
        "Undo",
        "source",
    )?;
    ensure_expected_size(
        copied_destination,
        operation.expected_destination_size,
        "Undo",
        "destination",
    )?;

    // The copy goes only after the kept source is proven to hold the same bytes.
    safe_fs::remove_verified_duplicate(copied_destination, original_source).map_err(|error| {
        OperationOutcome::blocked(format!(
            "Undo blocked: copied destination was kept: {error}"
        ))
    })
}
