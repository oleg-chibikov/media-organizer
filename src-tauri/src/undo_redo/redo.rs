use std::{fs, io, path::Path};

use super::outcome::{ensure_expected_size, OperationOutcome};
use crate::models::{HistoryOperation, HistoryOperationKind, MoveStrategy};

pub(super) fn execute_redo_operation(operation: &HistoryOperation) -> OperationOutcome {
    let result = match operation.move_strategy {
        MoveStrategy::Rename => redo_transfer(
            operation,
            |source, destination| fs::rename(source, destination),
            "Redo failed: could not move file back to processed destination",
        ),
        MoveStrategy::CopyKeepSource => redo_transfer(
            operation,
            |source, destination| fs::copy(source, destination).map(|_| ()),
            "Redo failed: could not copy file to recorded destination",
        ),
        MoveStrategy::NotAttempted => Err(OperationOutcome::blocked(
            "Operation strategy is not redoable.",
        )),
    };
    result.map_or_else(|outcome| outcome, |()| OperationOutcome::succeeded())
}

fn redo_transfer(
    operation: &HistoryOperation,
    transfer: impl Fn(&Path, &Path) -> io::Result<()>,
    failure_message: &str,
) -> Result<(), OperationOutcome> {
    let (redo_source, redo_destination) = redo_paths_for_operation(operation);

    if !redo_source.is_file() {
        return Err(OperationOutcome::blocked(
            "Redo blocked: source file is missing.",
        ));
    }
    if redo_destination.exists() {
        return Err(OperationOutcome::blocked(
            "Redo blocked: recorded destination path already exists.",
        ));
    }
    ensure_expected_size(
        redo_source,
        expected_redo_source_size(operation),
        "Redo",
        "source",
    )?;

    if let Some(parent) = redo_destination.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            OperationOutcome::failed(format!(
                "Redo failed: could not create destination parent directory: {error}"
            ))
        })?;
    }

    transfer(redo_source, redo_destination)
        .map_err(|error| OperationOutcome::failed(format!("{failure_message}: {error}")))?;
    Ok(())
}

/// Legacy runs without undo records are redone straight from their process records.
pub(super) fn expected_redo_source_size(operation: &HistoryOperation) -> Option<u64> {
    match operation.operation_kind {
        HistoryOperationKind::Process => operation
            .expected_source_size
            .or(operation.expected_destination_size),
        _ => operation
            .expected_destination_size
            .or(operation.expected_source_size),
    }
}

fn redo_paths_for_operation(operation: &HistoryOperation) -> (&Path, &Path) {
    match operation.operation_kind {
        HistoryOperationKind::Process => (
            Path::new(&operation.source_path),
            Path::new(&operation.destination_path),
        ),
        _ => (
            Path::new(&operation.destination_path),
            Path::new(&operation.source_path),
        ),
    }
}
