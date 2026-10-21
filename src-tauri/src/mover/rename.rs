use std::{fs, io, path::Path};

use super::{
    copy_fallback::execute_copy_verify_keep_source,
    outcome::{attempted_result, skipped_operation},
    MoveOperation,
};
use crate::{
    models::{MoveResult, MoveResultStatus, MoveStrategy},
    safe_fs,
};

pub(crate) fn execute_move_with_rename(operation: &MoveOperation) -> MoveResult {
    execute_move_with_rename_internal(operation, safe_fs::move_file_no_replace)
}

/// Moves without overwriting. Volumes that can't move in place get a verified copy that
/// keeps the source.
pub(super) fn execute_move_with_rename_internal<F>(
    operation: &MoveOperation,
    move_fn: F,
) -> MoveResult
where
    F: Fn(&Path, &Path) -> io::Result<()>,
{
    let source = Path::new(&operation.source_path);
    let destination = Path::new(&operation.destination_path);
    let failed = |error: String| {
        attempted_result(
            operation,
            MoveStrategy::Rename,
            MoveResultStatus::Failed,
            Some(error),
        )
    };

    if safe_fs::path_is_occupied(destination) {
        return skipped_operation(
            operation,
            "Destination already exists. Refusing to overwrite.",
        );
    }

    if !safe_fs::is_plain_file(source) {
        return failed("Source file no longer exists at execution time.".to_string());
    }

    if let Some(parent) = destination.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return failed(format!(
                "Failed to create destination directory before rename: {error}"
            ));
        }
    }

    match move_fn(source, destination) {
        Ok(()) => attempted_result(
            operation,
            MoveStrategy::Rename,
            MoveResultStatus::Moved,
            None,
        ),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => skipped_operation(
            operation,
            "Destination appeared right before the move. Refusing to overwrite.",
        ),
        Err(error) if needs_copy_fallback(&error) => {
            execute_copy_verify_keep_source(operation, source, destination)
        }
        Err(error) => failed(format!("No-overwrite move failed: {error}")),
    }
}

fn needs_copy_fallback(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::CrossesDevices | io::ErrorKind::Unsupported
    )
}
